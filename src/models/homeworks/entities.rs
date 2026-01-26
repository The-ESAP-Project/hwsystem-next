use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 作业用户状态（学生视角）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../frontend/src/types/generated/homework.ts")]
pub enum HomeworkUserStatus {
    /// 待完成（未提交）
    Pending,
    /// 已提交待批改
    Submitted,
    /// 已批改
    Graded,
}

/// 截止日期过滤器
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../frontend/src/types/generated/homework.ts")]
#[derive(Default)]
pub enum DeadlineFilter {
    /// 未过期
    Active,
    /// 已过期
    Expired,
    /// 全部
    #[default]
    All,
}


#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/types/generated/homework.ts")]
pub struct Homework {
    // 唯一 ID
    pub id: i64,
    // 关联的班级 ID
    pub class_id: i64,
    // 作业标题
    pub title: String,
    // 作业描述
    pub description: Option<String>,
    // 作业最高分数
    pub max_score: f64,
    // 作业截止时间
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    // 是否允许迟交
    pub allow_late: bool,
    // 创建者 ID
    pub created_by: i64,
    // 作业创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    // 作业更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
