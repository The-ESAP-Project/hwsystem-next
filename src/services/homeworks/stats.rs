use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use std::collections::{HashMap, HashSet};

use super::HomeworkService;
use crate::middlewares::RequireJWT;
use crate::models::class_users::entities::ClassUserRole;
use crate::models::homeworks::stats_responses::{
    HomeworkStatsResponse, ScoreRange, ScoreStats, UnsubmittedStudent,
};
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode};

pub async fn get_homework_stats(
    service: &HomeworkService,
    request: &HttpRequest,
    homework_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request);

    // 获取当前用户
    let user_id = match RequireJWT::extract_user_id(request) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::error_empty(
                ErrorCode::Unauthorized,
                "无法获取用户信息",
            )));
        }
    };

    // 获取作业信息
    let homework = match storage.get_homework_by_id(homework_id).await {
        Ok(Some(hw)) => hw,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
                ErrorCode::HomeworkNotFound,
                "作业不存在",
            )));
        }
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("查询作业失败: {e}"),
                )),
            );
        }
    };

    let class_id = homework.class_id;

    // 获取用户角色
    let user_role = RequireJWT::extract_user_role(request);

    // Admin 直接放行，跳过班级成员检查
    if user_role != Some(UserRole::Admin) {
        // 非 Admin 用户需要验证班级成员资格
        let class_user = match storage
            .get_class_user_by_user_id_and_class_id(user_id, class_id)
            .await
        {
            Ok(Some(cu)) => cu,
            Ok(None) => {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                    ErrorCode::ClassPermissionDenied,
                    "您不是该班级成员",
                )));
            }
            Err(e) => {
                return Ok(
                    HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                        ErrorCode::InternalServerError,
                        format!("查询班级成员失败: {e}"),
                    )),
                );
            }
        };

        // 验证是教师或课代表
        if class_user.role != ClassUserRole::Teacher
            && class_user.role != ClassUserRole::ClassRepresentative
        {
            return Ok(HttpResponse::Forbidden().json(ApiResponse::error_empty(
                ErrorCode::ClassPermissionDenied,
                "只有教师或课代表可以查看统计",
            )));
        }
    }

    // 获取班级所有成员（不分页）
    let class_users = match storage.list_all_class_users(class_id).await {
        Ok(users) => users,
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("查询班级成员失败: {e}"),
                )),
            );
        }
    };

    // 统计需要提交作业的成员（排除教师）
    let students: Vec<_> = class_users
        .iter()
        .filter(|cu| cu.role != ClassUserRole::Teacher)
        .collect();
    let total_students = students.len() as i64;
    let student_ids: HashSet<i64> = students.iter().map(|cu| cu.user_id).collect();

    // 获取该作业的所有提交（不分页）
    let submissions = match storage.list_all_submissions_by_homework(homework_id).await {
        Ok(subs) => subs,
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("查询提交失败: {e}"),
                )),
            );
        }
    };

    // 只统计学生的提交，并为每个学生只保留最新版本
    let mut latest_submissions: HashMap<
        i64,
        &crate::models::submissions::responses::SubmissionListItem,
    > = HashMap::new();
    for submission in &submissions {
        if !student_ids.contains(&submission.creator_id) {
            continue;
        }
        let entry = latest_submissions
            .entry(submission.creator_id)
            .or_insert(submission);
        if submission.version > entry.version {
            *entry = submission;
        }
    }

    let student_submissions: Vec<_> = latest_submissions.values().collect();

    let submitted_count = student_submissions.len() as i64;
    let late_count = student_submissions.iter().filter(|s| s.is_late).count() as i64;

    // 收集已提交学生的 ID
    let submitted_student_ids: HashSet<i64> =
        student_submissions.iter().map(|s| s.creator_id).collect();

    // 获取所有提交的评分
    let mut graded_count = 0i64;
    let mut scores: Vec<f64> = Vec::new();

    for submission in &student_submissions {
        if let Ok(Some(grade)) = storage.get_grade_by_submission_id(submission.id).await {
            graded_count += 1;
            scores.push(grade.score);
        }
    }

    // 计算分数统计
    let score_stats = if !scores.is_empty() {
        let sum: f64 = scores.iter().sum();
        let average = sum / scores.len() as f64;
        let max = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min = scores.iter().cloned().fold(f64::INFINITY, f64::min);
        Some(ScoreStats {
            average: (average * 100.0).round() / 100.0, // 保留两位小数
            max,
            min,
        })
    } else {
        None
    };

    // 计算分数分布（基于作业满分换算为百分比）
    let max_score = homework.max_score;
    let score_distribution = calculate_score_distribution(&scores, max_score);

    // 计算提交率
    let submission_rate = if total_students > 0 {
        (submitted_count as f64 / total_students as f64 * 100.0 * 100.0).round() / 100.0
    } else {
        0.0
    };

    // 获取未提交学生列表
    let mut unsubmitted_students: Vec<UnsubmittedStudent> = Vec::new();
    for student in &students {
        if !submitted_student_ids.contains(&student.user_id)
            && let Ok(Some(user)) = storage.get_user_by_id(student.user_id).await
        {
            unsubmitted_students.push(UnsubmittedStudent {
                id: user.id,
                username: user.username,
                display_name: user.display_name,
                avatar_url: user.avatar_url,
            });
        }
    }

    let response = HomeworkStatsResponse {
        homework_id,
        total_students,
        submitted_count,
        graded_count,
        late_count,
        submission_rate,
        score_stats,
        score_distribution,
        unsubmitted_students,
    };

    Ok(HttpResponse::Ok().json(ApiResponse::success(response, "查询成功")))
}

/// 计算分数分布
fn calculate_score_distribution(scores: &[f64], max_score: f64) -> Vec<ScoreRange> {
    if max_score <= 0.0 {
        return vec![];
    }

    // 将分数换算为百分比后分组
    let mut distribution: HashMap<&str, i64> = HashMap::new();
    distribution.insert("90-100", 0);
    distribution.insert("80-89", 0);
    distribution.insert("70-79", 0);
    distribution.insert("60-69", 0);
    distribution.insert("0-59", 0);

    for &score in scores {
        let percentage = (score / max_score) * 100.0;
        let range = if percentage >= 90.0 {
            "90-100"
        } else if percentage >= 80.0 {
            "80-89"
        } else if percentage >= 70.0 {
            "70-79"
        } else if percentage >= 60.0 {
            "60-69"
        } else {
            "0-59"
        };
        *distribution.get_mut(range).unwrap() += 1;
    }

    // 按顺序返回
    vec![
        ScoreRange {
            range: "90-100".to_string(),
            count: distribution["90-100"],
        },
        ScoreRange {
            range: "80-89".to_string(),
            count: distribution["80-89"],
        },
        ScoreRange {
            range: "70-79".to_string(),
            count: distribution["70-79"],
        },
        ScoreRange {
            range: "60-69".to_string(),
            count: distribution["60-69"],
        },
        ScoreRange {
            range: "0-59".to_string(),
            count: distribution["0-59"],
        },
    ]
}
