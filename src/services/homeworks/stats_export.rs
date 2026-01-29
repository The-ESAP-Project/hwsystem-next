//! 作业统计导出服务

use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use chrono::Utc;
use rust_xlsxwriter::{Format, Workbook, Worksheet};
use std::collections::{HashMap, HashSet};
use tracing::error;

use super::HomeworkService;
use crate::middlewares::RequireJWT;
use crate::models::class_users::entities::ClassUserRole;
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode};

/// 学生明细信息
struct StudentDetail {
    display_name: String,
    username: String,
    submitted: bool,
    score: Option<f64>,
    submitted_at: Option<String>,
    is_late: bool,
}

/// 导出作业统计报表
pub async fn export_homework_stats(
    service: &HomeworkService,
    request: &HttpRequest,
    homework_id: i64,
) -> ActixResult<HttpResponse> {
    let storage = service.get_storage(request)?;

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
    let mut show_scores = true; // 默认显示分数

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
                "只有教师或课代表可以导出统计",
            )));
        }

        // 课代表不显示具体分数
        if class_user.role == ClassUserRole::ClassRepresentative {
            show_scores = false;
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

    // 收集已提交学生的 ID（当前未使用，保留以备后续扩展）
    let _submitted_student_ids: HashSet<i64> =
        student_submissions.iter().map(|s| s.creator_id).collect();

    // 获取所有提交的评分
    let mut graded_count = 0i64;
    let mut scores: Vec<f64> = Vec::new();
    let mut submission_grades: HashMap<i64, f64> = HashMap::new(); // submission_id -> score

    for submission in &student_submissions {
        if let Ok(Some(grade)) = storage.get_grade_by_submission_id(submission.id).await {
            graded_count += 1;
            scores.push(grade.score);
            submission_grades.insert(submission.id, grade.score);
        }
    }

    // 计算分数统计
    let (avg_score, max_score_val, min_score_val) = if !scores.is_empty() {
        let sum: f64 = scores.iter().sum();
        let average = (sum / scores.len() as f64 * 100.0).round() / 100.0;
        let max = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min = scores.iter().cloned().fold(f64::INFINITY, f64::min);
        (Some(average), Some(max), Some(min))
    } else {
        (None, None, None)
    };

    // 计算分数分布
    let max_score = homework.max_score;
    let score_distribution = calculate_score_distribution(&scores, max_score);

    // 计算提交率
    let submission_rate = if total_students > 0 {
        (submitted_count as f64 / total_students as f64 * 100.0 * 100.0).round() / 100.0
    } else {
        0.0
    };

    // 构建学生明细数据
    let mut student_details: Vec<StudentDetail> = Vec::new();
    for student in &students {
        if let Ok(Some(user)) = storage.get_user_by_id(student.user_id).await {
            let submission = latest_submissions.get(&student.user_id);
            let (submitted, score, submitted_at, is_late) = if let Some(sub) = submission {
                let score = submission_grades.get(&sub.id).copied();
                (true, score, Some(sub.submitted_at.clone()), sub.is_late)
            } else {
                (false, None, None, false)
            };

            student_details.push(StudentDetail {
                display_name: user.display_name.unwrap_or_else(|| user.username.clone()),
                username: user.username,
                submitted,
                score,
                submitted_at,
                is_late,
            });
        }
    }

    // 生成 XLSX
    let xlsx_result = generate_xlsx(
        &homework.title,
        total_students,
        submitted_count,
        graded_count,
        late_count,
        submission_rate,
        avg_score,
        max_score_val,
        min_score_val,
        max_score,
        &score_distribution,
        &student_details,
        show_scores,
    );

    match xlsx_result {
        Ok(buffer) => {
            let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
            let filename = format!("homework_{homework_id}_stats_{timestamp}.xlsx");

            Ok(HttpResponse::Ok()
                .content_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
                .insert_header((
                    "Content-Disposition",
                    format!("attachment; filename=\"{filename}\""),
                ))
                .body(buffer))
        }
        Err(e) => {
            error!("生成 XLSX 失败: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("生成报表失败: {e}"),
                )),
            )
        }
    }
}

/// 计算分数分布
fn calculate_score_distribution(scores: &[f64], max_score: f64) -> Vec<(String, i64)> {
    if max_score <= 0.0 {
        return vec![];
    }

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

    vec![
        ("90-100".to_string(), distribution["90-100"]),
        ("80-89".to_string(), distribution["80-89"]),
        ("70-79".to_string(), distribution["70-79"]),
        ("60-69".to_string(), distribution["60-69"]),
        ("0-59".to_string(), distribution["0-59"]),
    ]
}

/// 生成 XLSX 文件
#[allow(clippy::too_many_arguments)]
fn generate_xlsx(
    homework_title: &str,
    total_students: i64,
    submitted_count: i64,
    graded_count: i64,
    late_count: i64,
    submission_rate: f64,
    avg_score: Option<f64>,
    max_score: Option<f64>,
    min_score: Option<f64>,
    homework_max_score: f64,
    score_distribution: &[(String, i64)],
    student_details: &[StudentDetail],
    show_scores: bool,
) -> Result<Vec<u8>, String> {
    let mut workbook = Workbook::new();

    // 格式定义
    let header_format = Format::new().set_bold();
    let title_format = Format::new().set_bold().set_font_size(14);

    // Sheet 1: 统计摘要
    let sheet1 = workbook
        .add_worksheet()
        .set_name("统计摘要")
        .map_err(|e| e.to_string())?;
    write_summary_sheet(
        sheet1,
        &header_format,
        &title_format,
        homework_title,
        total_students,
        submitted_count,
        graded_count,
        late_count,
        submission_rate,
        avg_score,
        max_score,
        min_score,
        homework_max_score,
        show_scores,
    )?;

    // Sheet 2: 分数分布
    let sheet2 = workbook
        .add_worksheet()
        .set_name("分数分布")
        .map_err(|e| e.to_string())?;
    write_distribution_sheet(
        sheet2,
        &header_format,
        score_distribution,
        graded_count,
        show_scores,
    )?;

    // Sheet 3: 学生明细
    let sheet3 = workbook
        .add_worksheet()
        .set_name("学生明细")
        .map_err(|e| e.to_string())?;
    write_details_sheet(sheet3, &header_format, student_details, show_scores)?;

    // 生成二进制数据
    workbook.save_to_buffer().map_err(|e| e.to_string())
}

/// 写入统计摘要 Sheet
#[allow(clippy::too_many_arguments)]
fn write_summary_sheet(
    sheet: &mut Worksheet,
    header_format: &Format,
    title_format: &Format,
    homework_title: &str,
    total_students: i64,
    submitted_count: i64,
    graded_count: i64,
    late_count: i64,
    submission_rate: f64,
    avg_score: Option<f64>,
    max_score: Option<f64>,
    min_score: Option<f64>,
    homework_max_score: f64,
    show_scores: bool,
) -> Result<(), String> {
    // 标题
    sheet
        .write_string_with_format(0, 0, "作业统计报表", title_format)
        .map_err(|e| e.to_string())?;

    // 表头
    sheet
        .write_string_with_format(2, 0, "项目", header_format)
        .map_err(|e| e.to_string())?;
    sheet
        .write_string_with_format(2, 1, "数值", header_format)
        .map_err(|e| e.to_string())?;

    // 数据
    let mut row = 3u32;

    sheet.write_string(row, 0, "作业标题").ok();
    sheet.write_string(row, 1, homework_title).ok();
    row += 1;

    sheet.write_string(row, 0, "满分").ok();
    sheet.write_number(row, 1, homework_max_score).ok();
    row += 1;

    sheet.write_string(row, 0, "班级学生总数").ok();
    sheet.write_number(row, 1, total_students as f64).ok();
    row += 1;

    sheet.write_string(row, 0, "已提交人数").ok();
    sheet.write_number(row, 1, submitted_count as f64).ok();
    row += 1;

    sheet.write_string(row, 0, "已批改人数").ok();
    sheet.write_number(row, 1, graded_count as f64).ok();
    row += 1;

    sheet.write_string(row, 0, "迟交人数").ok();
    sheet.write_number(row, 1, late_count as f64).ok();
    row += 1;

    sheet.write_string(row, 0, "提交率").ok();
    sheet
        .write_string(row, 1, format!("{submission_rate}%"))
        .ok();
    row += 1;

    // 分数统计（根据权限显示）
    sheet.write_string(row, 0, "平均分").ok();
    if show_scores {
        if let Some(avg) = avg_score {
            sheet.write_number(row, 1, avg).ok();
        } else {
            sheet.write_string(row, 1, "-").ok();
        }
    } else {
        sheet.write_string(row, 1, "***").ok();
    }
    row += 1;

    sheet.write_string(row, 0, "最高分").ok();
    if show_scores {
        if let Some(max) = max_score {
            sheet.write_number(row, 1, max).ok();
        } else {
            sheet.write_string(row, 1, "-").ok();
        }
    } else {
        sheet.write_string(row, 1, "***").ok();
    }
    row += 1;

    sheet.write_string(row, 0, "最低分").ok();
    if show_scores {
        if let Some(min) = min_score {
            sheet.write_number(row, 1, min).ok();
        } else {
            sheet.write_string(row, 1, "-").ok();
        }
    } else {
        sheet.write_string(row, 1, "***").ok();
    }

    // 设置列宽
    sheet.set_column_width(0, 20).ok();
    sheet.set_column_width(1, 30).ok();

    Ok(())
}

/// 写入分数分布 Sheet
fn write_distribution_sheet(
    sheet: &mut Worksheet,
    header_format: &Format,
    score_distribution: &[(String, i64)],
    graded_count: i64,
    show_scores: bool,
) -> Result<(), String> {
    // 表头
    sheet
        .write_string_with_format(0, 0, "分数区间", header_format)
        .map_err(|e| e.to_string())?;
    sheet
        .write_string_with_format(0, 1, "人数", header_format)
        .map_err(|e| e.to_string())?;
    sheet
        .write_string_with_format(0, 2, "占比", header_format)
        .map_err(|e| e.to_string())?;

    if !show_scores {
        // 课代表无法查看分数分布详情
        sheet.write_string(1, 0, "无权限查看").ok();
        sheet.write_string(1, 1, "***").ok();
        sheet.write_string(1, 2, "***").ok();
    } else {
        // 数据
        for (row, (range, count)) in score_distribution.iter().enumerate() {
            let row = (row + 1) as u32;
            sheet.write_string(row, 0, range).ok();
            sheet.write_number(row, 1, *count as f64).ok();

            let percentage = if graded_count > 0 {
                (*count as f64 / graded_count as f64 * 100.0 * 100.0).round() / 100.0
            } else {
                0.0
            };
            sheet.write_string(row, 2, format!("{percentage}%")).ok();
        }
    }

    // 设置列宽
    sheet.set_column_width(0, 15).ok();
    sheet.set_column_width(1, 10).ok();
    sheet.set_column_width(2, 10).ok();

    Ok(())
}

/// 写入学生明细 Sheet
fn write_details_sheet(
    sheet: &mut Worksheet,
    header_format: &Format,
    student_details: &[StudentDetail],
    show_scores: bool,
) -> Result<(), String> {
    // 表头
    let headers = ["姓名", "用户名", "提交状态", "分数", "提交时间", "迟交"];
    for (col, header) in headers.iter().enumerate() {
        sheet
            .write_string_with_format(0, col as u16, *header, header_format)
            .map_err(|e| e.to_string())?;
    }

    // 数据
    for (row, student) in student_details.iter().enumerate() {
        let row = (row + 1) as u32;

        sheet.write_string(row, 0, &student.display_name).ok();
        sheet.write_string(row, 1, &student.username).ok();

        // 提交状态
        let status = if student.submitted {
            "已提交"
        } else {
            "未提交"
        };
        sheet.write_string(row, 2, status).ok();

        // 分数（根据权限显示）
        if show_scores {
            if let Some(score) = student.score {
                sheet.write_number(row, 3, score).ok();
            } else if student.submitted {
                sheet.write_string(row, 3, "待批改").ok();
            } else {
                sheet.write_string(row, 3, "-").ok();
            }
        } else {
            sheet.write_string(row, 3, "***").ok();
        }

        // 提交时间
        if let Some(ref time) = student.submitted_at {
            sheet.write_string(row, 4, time).ok();
        } else {
            sheet.write_string(row, 4, "-").ok();
        }

        // 迟交
        let late = if student.is_late { "是" } else { "-" };
        sheet.write_string(row, 5, late).ok();
    }

    // 设置列宽
    sheet.set_column_width(0, 15).ok();
    sheet.set_column_width(1, 15).ok();
    sheet.set_column_width(2, 10).ok();
    sheet.set_column_width(3, 10).ok();
    sheet.set_column_width(4, 20).ok();
    sheet.set_column_width(5, 8).ok();

    Ok(())
}
