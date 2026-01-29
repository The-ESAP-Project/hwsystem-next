//! 班级报表导出服务

use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use chrono::Utc;
use rust_xlsxwriter::{Format, Workbook, Worksheet};
use std::collections::{HashMap, HashSet};
use tracing::error;

use super::ClassService;
use crate::middlewares::RequireJWT;
use crate::models::class_users::entities::ClassUserRole;
use crate::models::users::entities::UserRole;
use crate::models::{ApiResponse, ErrorCode};

/// 学生作业状态
#[derive(Debug, Clone)]
enum StudentHomeworkStatus {
    /// 未提交
    NotSubmitted,
    /// 已提交待批改
    Submitted,
    /// 已评分
    Graded(f64),
}

/// 作业汇总数据
struct HomeworkSummary {
    title: String,
    deadline: Option<String>,
    submitted_count: i64,
    total_students: i64,
    graded_count: i64,
    avg_score: Option<f64>,
}

/// 学生明细数据
struct StudentDetail {
    display_name: String,
    username: String,
    homework_statuses: Vec<StudentHomeworkStatus>,
    total_submitted: i64,
    total_homeworks: i64,
    avg_score: Option<f64>,
}

/// 导出班级报表
pub async fn export_class_report(
    service: &ClassService,
    request: &HttpRequest,
    class_id: i64,
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

    // 获取班级信息
    let class = match storage.get_class_by_id(class_id).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::error_empty(
                ErrorCode::ClassNotFound,
                "班级不存在",
            )));
        }
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("查询班级失败: {e}"),
                )),
            );
        }
    };

    // 获取用户角色
    let user_role = RequireJWT::extract_user_role(request);
    let mut show_scores = true;

    // Admin 直接放行
    if user_role != Some(UserRole::Admin) {
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
                "只有教师或课代表可以导出报表",
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

    // 获取班级所有作业（不分页）
    let homeworks = match storage.list_all_homeworks_by_class(class_id).await {
        Ok(hws) => hws,
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::error_empty(
                    ErrorCode::InternalServerError,
                    format!("查询作业失败: {e}"),
                )),
            );
        }
    };

    let total_homeworks = homeworks.len() as i64;

    // 收集所有作业的提交和评分数据
    // homework_id -> (user_id -> StudentHomeworkStatus)
    let mut homework_submissions: HashMap<i64, HashMap<i64, StudentHomeworkStatus>> =
        HashMap::new();
    let mut homework_summaries: Vec<HomeworkSummary> = Vec::new();

    for homework in &homeworks {
        let hw_id = homework.id;

        // 获取该作业的所有提交（不分页）
        let submissions = match storage.list_all_submissions_by_homework(hw_id).await {
            Ok(subs) => subs,
            Err(e) => {
                error!("查询作业 {} 的提交失败: {}", hw_id, e);
                continue;
            }
        };

        // 为每个学生只保留最新版本的提交
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

        // 获取评分信息并构建状态映射
        let mut user_statuses: HashMap<i64, StudentHomeworkStatus> = HashMap::new();
        let mut graded_count = 0i64;
        let mut scores: Vec<f64> = Vec::new();

        for (&user_id, submission) in &latest_submissions {
            if let Ok(Some(grade)) = storage.get_grade_by_submission_id(submission.id).await {
                user_statuses.insert(user_id, StudentHomeworkStatus::Graded(grade.score));
                graded_count += 1;
                scores.push(grade.score);
            } else {
                user_statuses.insert(user_id, StudentHomeworkStatus::Submitted);
            }
        }

        let submitted_count = latest_submissions.len() as i64;
        let avg_score = if !scores.is_empty() {
            Some((scores.iter().sum::<f64>() / scores.len() as f64 * 100.0).round() / 100.0)
        } else {
            None
        };

        homework_submissions.insert(hw_id, user_statuses);

        homework_summaries.push(HomeworkSummary {
            title: homework.title.clone(),
            deadline: homework.deadline.as_ref().map(|d| d.to_string()),
            submitted_count,
            total_students,
            graded_count,
            avg_score,
        });
    }

    // 构建学生明细数据
    let mut student_details: Vec<StudentDetail> = Vec::new();
    for student in &students {
        if let Ok(Some(user)) = storage.get_user_by_id(student.user_id).await {
            let mut statuses: Vec<StudentHomeworkStatus> = Vec::new();
            let mut total_submitted = 0i64;
            let mut score_sum = 0.0f64;
            let mut graded_count = 0i64;

            for homework in &homeworks {
                let status = homework_submissions
                    .get(&homework.id)
                    .and_then(|m| m.get(&student.user_id))
                    .cloned()
                    .unwrap_or(StudentHomeworkStatus::NotSubmitted);

                match &status {
                    StudentHomeworkStatus::NotSubmitted => {}
                    StudentHomeworkStatus::Submitted => {
                        total_submitted += 1;
                    }
                    StudentHomeworkStatus::Graded(score) => {
                        total_submitted += 1;
                        score_sum += score;
                        graded_count += 1;
                    }
                }
                statuses.push(status);
            }

            let avg_score = if graded_count > 0 {
                Some((score_sum / graded_count as f64 * 100.0).round() / 100.0)
            } else {
                None
            };

            student_details.push(StudentDetail {
                display_name: user.display_name.unwrap_or_else(|| user.username.clone()),
                username: user.username,
                homework_statuses: statuses,
                total_submitted,
                total_homeworks,
                avg_score,
            });
        }
    }

    // 计算整体平均提交率
    let total_submission_count: i64 = homework_summaries.iter().map(|h| h.submitted_count).sum();
    let avg_submission_rate = if total_students > 0 && total_homeworks > 0 {
        (total_submission_count as f64 / (total_students * total_homeworks) as f64 * 100.0 * 100.0)
            .round()
            / 100.0
    } else {
        0.0
    };

    // 生成 XLSX
    let homework_titles: Vec<String> = homeworks.iter().map(|h| h.title.clone()).collect();

    let xlsx_result = generate_xlsx(
        &class.name,
        total_students,
        total_homeworks,
        avg_submission_rate,
        &homework_summaries,
        &student_details,
        &homework_titles,
        show_scores,
    );

    match xlsx_result {
        Ok(buffer) => {
            let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
            let filename = format!("class_{class_id}_report_{timestamp}.xlsx");

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

/// 生成 XLSX 文件
#[allow(clippy::too_many_arguments)]
fn generate_xlsx(
    class_name: &str,
    total_students: i64,
    total_homeworks: i64,
    avg_submission_rate: f64,
    homework_summaries: &[HomeworkSummary],
    student_details: &[StudentDetail],
    homework_titles: &[String],
    show_scores: bool,
) -> Result<Vec<u8>, String> {
    let mut workbook = Workbook::new();

    // 格式定义
    let header_format = Format::new().set_bold();
    let title_format = Format::new().set_bold().set_font_size(14);

    // Sheet 1: 班级概览
    let sheet1 = workbook
        .add_worksheet()
        .set_name("班级概览")
        .map_err(|e| e.to_string())?;
    write_overview_sheet(
        sheet1,
        &header_format,
        &title_format,
        class_name,
        total_students,
        total_homeworks,
        avg_submission_rate,
    )?;

    // Sheet 2: 作业汇总
    let sheet2 = workbook
        .add_worksheet()
        .set_name("作业汇总")
        .map_err(|e| e.to_string())?;
    write_homework_summary_sheet(sheet2, &header_format, homework_summaries, show_scores)?;

    // Sheet 3: 学生明细
    let sheet3 = workbook
        .add_worksheet()
        .set_name("学生明细")
        .map_err(|e| e.to_string())?;
    write_student_details_sheet(
        sheet3,
        &header_format,
        student_details,
        homework_titles,
        show_scores,
    )?;

    // 生成二进制数据
    workbook.save_to_buffer().map_err(|e| e.to_string())
}

/// 写入班级概览 Sheet
fn write_overview_sheet(
    sheet: &mut Worksheet,
    header_format: &Format,
    title_format: &Format,
    class_name: &str,
    total_students: i64,
    total_homeworks: i64,
    avg_submission_rate: f64,
) -> Result<(), String> {
    // 标题
    sheet
        .write_string_with_format(0, 0, "班级报表", title_format)
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

    sheet.write_string(row, 0, "班级名称").ok();
    sheet.write_string(row, 1, class_name).ok();
    row += 1;

    sheet.write_string(row, 0, "学生总数").ok();
    sheet.write_number(row, 1, total_students as f64).ok();
    row += 1;

    sheet.write_string(row, 0, "作业总数").ok();
    sheet.write_number(row, 1, total_homeworks as f64).ok();
    row += 1;

    sheet.write_string(row, 0, "整体平均提交率").ok();
    sheet
        .write_string(row, 1, format!("{avg_submission_rate}%"))
        .ok();

    // 设置列宽
    sheet.set_column_width(0, 20).ok();
    sheet.set_column_width(1, 30).ok();

    Ok(())
}

/// 写入作业汇总 Sheet
fn write_homework_summary_sheet(
    sheet: &mut Worksheet,
    header_format: &Format,
    homework_summaries: &[HomeworkSummary],
    show_scores: bool,
) -> Result<(), String> {
    // 表头
    let headers = [
        "作业标题",
        "截止时间",
        "提交人数",
        "提交率",
        "批改人数",
        "平均分",
    ];
    for (col, header) in headers.iter().enumerate() {
        sheet
            .write_string_with_format(0, col as u16, *header, header_format)
            .map_err(|e| e.to_string())?;
    }

    // 数据
    for (row, summary) in homework_summaries.iter().enumerate() {
        let row = (row + 1) as u32;

        sheet.write_string(row, 0, &summary.title).ok();

        // 截止时间
        if let Some(ref deadline) = summary.deadline {
            sheet.write_string(row, 1, deadline).ok();
        } else {
            sheet.write_string(row, 1, "-").ok();
        }

        // 提交人数
        sheet
            .write_number(row, 2, summary.submitted_count as f64)
            .ok();

        // 提交率
        let rate = if summary.total_students > 0 {
            (summary.submitted_count as f64 / summary.total_students as f64 * 100.0 * 100.0).round()
                / 100.0
        } else {
            0.0
        };
        sheet.write_string(row, 3, format!("{rate}%")).ok();

        // 批改人数
        sheet.write_number(row, 4, summary.graded_count as f64).ok();

        // 平均分
        if show_scores {
            if let Some(avg) = summary.avg_score {
                sheet.write_number(row, 5, avg).ok();
            } else {
                sheet.write_string(row, 5, "-").ok();
            }
        } else {
            sheet.write_string(row, 5, "***").ok();
        }
    }

    // 设置列宽
    sheet.set_column_width(0, 30).ok();
    sheet.set_column_width(1, 20).ok();
    sheet.set_column_width(2, 12).ok();
    sheet.set_column_width(3, 12).ok();
    sheet.set_column_width(4, 12).ok();
    sheet.set_column_width(5, 12).ok();

    Ok(())
}

/// 写入学生明细 Sheet（矩阵形式）
fn write_student_details_sheet(
    sheet: &mut Worksheet,
    header_format: &Format,
    student_details: &[StudentDetail],
    homework_titles: &[String],
    show_scores: bool,
) -> Result<(), String> {
    // 表头：姓名 | 用户名 | 作业1 | 作业2 | ... | 总提交数 | 平均分
    let mut col: u16 = 0;

    sheet
        .write_string_with_format(0, col, "姓名", header_format)
        .map_err(|e| e.to_string())?;
    col += 1;

    sheet
        .write_string_with_format(0, col, "用户名", header_format)
        .map_err(|e| e.to_string())?;
    col += 1;

    // 作业列
    for title in homework_titles {
        // 截断过长的标题
        let display_title = if title.len() > 15 {
            format!("{}...", &title[..12])
        } else {
            title.clone()
        };
        sheet
            .write_string_with_format(0, col, &display_title, header_format)
            .map_err(|e| e.to_string())?;
        col += 1;
    }

    sheet
        .write_string_with_format(0, col, "总提交数", header_format)
        .map_err(|e| e.to_string())?;
    col += 1;

    sheet
        .write_string_with_format(0, col, "平均分", header_format)
        .map_err(|e| e.to_string())?;

    // 数据行
    for (row, student) in student_details.iter().enumerate() {
        let row = (row + 1) as u32;
        let mut col: u16 = 0;

        sheet.write_string(row, col, &student.display_name).ok();
        col += 1;

        sheet.write_string(row, col, &student.username).ok();
        col += 1;

        // 每个作业的状态
        for status in &student.homework_statuses {
            let cell_value = match status {
                StudentHomeworkStatus::NotSubmitted => "-".to_string(),
                StudentHomeworkStatus::Submitted => "✓".to_string(),
                StudentHomeworkStatus::Graded(score) => {
                    if show_scores {
                        format!("{score}")
                    } else {
                        "✓".to_string()
                    }
                }
            };
            sheet.write_string(row, col, &cell_value).ok();
            col += 1;
        }

        // 总提交数
        sheet
            .write_string(
                row,
                col,
                format!("{}/{}", student.total_submitted, student.total_homeworks),
            )
            .ok();
        col += 1;

        // 平均分
        if show_scores {
            if let Some(avg) = student.avg_score {
                sheet.write_number(row, col, avg).ok();
            } else {
                sheet.write_string(row, col, "-").ok();
            }
        } else {
            sheet.write_string(row, col, "***").ok();
        }
    }

    // 设置列宽
    sheet.set_column_width(0, 15).ok(); // 姓名
    sheet.set_column_width(1, 15).ok(); // 用户名

    // 作业列宽度
    for i in 0..homework_titles.len() {
        sheet.set_column_width((i + 2) as u16, 10).ok();
    }

    // 总提交数和平均分列
    let last_col = (homework_titles.len() + 2) as u16;
    sheet.set_column_width(last_col, 12).ok();
    sheet.set_column_width(last_col + 1, 10).ok();

    Ok(())
}
