use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ==================== 用户表 ====================
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Users::Username)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Users::Email)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Users::PasswordHash).string().not_null())
                    .col(ColumnDef::new(Users::DisplayName).string().null())
                    .col(
                        ColumnDef::new(Users::Role)
                            .string()
                            .not_null()
                            .default("user"),
                    )
                    .col(
                        ColumnDef::new(Users::Status)
                            .string()
                            .not_null()
                            .default("active"),
                    )
                    .col(ColumnDef::new(Users::AvatarUrl).string().null())
                    .col(ColumnDef::new(Users::LastLogin).big_integer().null())
                    .col(ColumnDef::new(Users::CreatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Users::UpdatedAt).big_integer().not_null())
                    .to_owned(),
            )
            .await?;

        // 用户表索引
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_users_username")
                    .table(Users::Table)
                    .col(Users::Username)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_users_email")
                    .table(Users::Table)
                    .col(Users::Email)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_users_role")
                    .table(Users::Table)
                    .col(Users::Role)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_users_status")
                    .table(Users::Table)
                    .col(Users::Status)
                    .to_owned(),
            )
            .await?;

        // ==================== 班级表 ====================
        manager
            .create_table(
                Table::create()
                    .table(Classes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Classes::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Classes::Name).string().not_null())
                    .col(ColumnDef::new(Classes::Description).text().null())
                    .col(ColumnDef::new(Classes::TeacherId).big_integer().not_null())
                    .col(
                        ColumnDef::new(Classes::InviteCode)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Classes::CreatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Classes::UpdatedAt).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Classes::Table, Classes::TeacherId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 班级表索引
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_classes_teacher_id")
                    .table(Classes::Table)
                    .col(Classes::TeacherId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_classes_invite_code")
                    .table(Classes::Table)
                    .col(Classes::InviteCode)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // ==================== 班级用户关联表 ====================
        manager
            .create_table(
                Table::create()
                    .table(ClassUsers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ClassUsers::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ClassUsers::ClassId).big_integer().not_null())
                    .col(ColumnDef::new(ClassUsers::UserId).big_integer().not_null())
                    .col(
                        ColumnDef::new(ClassUsers::Role)
                            .string()
                            .not_null()
                            .default("student"),
                    )
                    .col(
                        ColumnDef::new(ClassUsers::JoinedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ClassUsers::Table, ClassUsers::ClassId)
                            .to(Classes::Table, Classes::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ClassUsers::Table, ClassUsers::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 班级用户唯一约束
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_class_users_unique")
                    .table(ClassUsers::Table)
                    .col(ClassUsers::ClassId)
                    .col(ClassUsers::UserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // 班级用户表索引
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_class_users_class_id")
                    .table(ClassUsers::Table)
                    .col(ClassUsers::ClassId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_class_users_user_id")
                    .table(ClassUsers::Table)
                    .col(ClassUsers::UserId)
                    .to_owned(),
            )
            .await?;

        // ==================== 作业表 ====================
        manager
            .create_table(
                Table::create()
                    .table(Homeworks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Homeworks::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Homeworks::ClassId).big_integer().not_null())
                    .col(ColumnDef::new(Homeworks::Title).string().not_null())
                    .col(ColumnDef::new(Homeworks::Description).text().null())
                    .col(
                        ColumnDef::new(Homeworks::MaxScore)
                            .double()
                            .not_null()
                            .default(100.0),
                    )
                    .col(ColumnDef::new(Homeworks::Deadline).big_integer().null())
                    .col(
                        ColumnDef::new(Homeworks::AllowLate)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Homeworks::CreatedBy)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Homeworks::CreatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Homeworks::UpdatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Homeworks::Table, Homeworks::ClassId)
                            .to(Classes::Table, Classes::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Homeworks::Table, Homeworks::CreatedBy)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 作业表索引
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_homeworks_class_id")
                    .table(Homeworks::Table)
                    .col(Homeworks::ClassId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_homeworks_created_by")
                    .table(Homeworks::Table)
                    .col(Homeworks::CreatedBy)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_homeworks_deadline")
                    .table(Homeworks::Table)
                    .col(Homeworks::Deadline)
                    .to_owned(),
            )
            .await?;

        // ==================== 提交表 ====================
        manager
            .create_table(
                Table::create()
                    .table(Submissions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Submissions::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Submissions::HomeworkId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Submissions::CreatorId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Submissions::Version)
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    .col(ColumnDef::new(Submissions::Content).text().null())
                    .col(
                        ColumnDef::new(Submissions::Status)
                            .string()
                            .not_null()
                            .default("pending"),
                    )
                    .col(
                        ColumnDef::new(Submissions::IsLate)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Submissions::SubmittedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Submissions::Table, Submissions::HomeworkId)
                            .to(Homeworks::Table, Homeworks::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Submissions::Table, Submissions::CreatorId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 提交表唯一约束：同一学生同一作业的每个版本唯一
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_submissions_unique_version")
                    .table(Submissions::Table)
                    .col(Submissions::HomeworkId)
                    .col(Submissions::CreatorId)
                    .col(Submissions::Version)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // 提交表索引
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_submissions_homework_id")
                    .table(Submissions::Table)
                    .col(Submissions::HomeworkId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_submissions_creator_id")
                    .table(Submissions::Table)
                    .col(Submissions::CreatorId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_submissions_status")
                    .table(Submissions::Table)
                    .col(Submissions::Status)
                    .to_owned(),
            )
            .await?;

        // 复合索引：查询某学生某作业的最新提交
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_submissions_hw_creator")
                    .table(Submissions::Table)
                    .col(Submissions::HomeworkId)
                    .col(Submissions::CreatorId)
                    .to_owned(),
            )
            .await?;

        // ==================== 评分表 ====================
        manager
            .create_table(
                Table::create()
                    .table(Grades::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Grades::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Grades::SubmissionId)
                            .big_integer()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Grades::GraderId).big_integer().not_null())
                    .col(ColumnDef::new(Grades::Score).double().not_null())
                    .col(ColumnDef::new(Grades::Comment).text().null())
                    .col(ColumnDef::new(Grades::GradedAt).big_integer().not_null())
                    .col(ColumnDef::new(Grades::UpdatedAt).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Grades::Table, Grades::SubmissionId)
                            .to(Submissions::Table, Submissions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Grades::Table, Grades::GraderId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // 评分表索引
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_grades_submission_id")
                    .table(Grades::Table)
                    .col(Grades::SubmissionId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_grades_grader_id")
                    .table(Grades::Table)
                    .col(Grades::GraderId)
                    .to_owned(),
            )
            .await?;

        // ==================== 文件表 ====================
        manager
            .create_table(
                Table::create()
                    .table(Files::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Files::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Files::UserId).big_integer().null())
                    .col(ColumnDef::new(Files::OriginalName).string().not_null())
                    .col(ColumnDef::new(Files::StoredName).string().not_null())
                    .col(ColumnDef::new(Files::FileType).string().not_null())
                    .col(ColumnDef::new(Files::FileSize).big_integer().not_null())
                    .col(ColumnDef::new(Files::FilePath).string().not_null())
                    .col(
                        ColumnDef::new(Files::DownloadToken)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Files::CitationCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(Files::CreatedAt).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Files::Table, Files::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // 文件表索引
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_files_user_id")
                    .table(Files::Table)
                    .col(Files::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_files_download_token")
                    .table(Files::Table)
                    .col(Files::DownloadToken)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // ==================== 作业附件关联表 ====================
        manager
            .create_table(
                Table::create()
                    .table(HomeworkFiles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(HomeworkFiles::HomeworkId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(HomeworkFiles::FileId)
                            .big_integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(HomeworkFiles::HomeworkId)
                            .col(HomeworkFiles::FileId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(HomeworkFiles::Table, HomeworkFiles::HomeworkId)
                            .to(Homeworks::Table, Homeworks::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(HomeworkFiles::Table, HomeworkFiles::FileId)
                            .to(Files::Table, Files::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // ==================== 提交附件关联表 ====================
        manager
            .create_table(
                Table::create()
                    .table(SubmissionFiles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SubmissionFiles::SubmissionId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SubmissionFiles::FileId)
                            .big_integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(SubmissionFiles::SubmissionId)
                            .col(SubmissionFiles::FileId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(SubmissionFiles::Table, SubmissionFiles::SubmissionId)
                            .to(Submissions::Table, Submissions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(SubmissionFiles::Table, SubmissionFiles::FileId)
                            .to(Files::Table, Files::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // ==================== 通知表 ====================
        manager
            .create_table(
                Table::create()
                    .table(Notifications::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Notifications::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Notifications::UserId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Notifications::Type).string().not_null())
                    .col(ColumnDef::new(Notifications::Title).string().not_null())
                    .col(ColumnDef::new(Notifications::Content).text().null())
                    .col(ColumnDef::new(Notifications::ReferenceType).string().null())
                    .col(
                        ColumnDef::new(Notifications::ReferenceId)
                            .big_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Notifications::IsRead)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Notifications::CreatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Notifications::Table, Notifications::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 通知表索引
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_notifications_user_id")
                    .table(Notifications::Table)
                    .col(Notifications::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_notifications_user_is_read")
                    .table(Notifications::Table)
                    .col(Notifications::UserId)
                    .col(Notifications::IsRead)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_notifications_created_at")
                    .table(Notifications::Table)
                    .col(Notifications::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 按照创建的相反顺序删除
        manager
            .drop_table(Table::drop().table(Notifications::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(SubmissionFiles::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(HomeworkFiles::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Files::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Grades::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Submissions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Homeworks::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ClassUsers::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Classes::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;
        Ok(())
    }
}

// ==================== 表标识符 ====================

#[derive(DeriveIden)]
enum Users {
    #[sea_orm(iden = "users")]
    Table,
    Id,
    Username,
    Email,
    PasswordHash,
    DisplayName,
    Role,
    Status,
    AvatarUrl,
    LastLogin,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Classes {
    #[sea_orm(iden = "classes")]
    Table,
    Id,
    Name,
    Description,
    TeacherId,
    InviteCode,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ClassUsers {
    #[sea_orm(iden = "class_users")]
    Table,
    Id,
    ClassId,
    UserId,
    Role,
    JoinedAt,
}

#[derive(DeriveIden)]
enum Homeworks {
    #[sea_orm(iden = "homeworks")]
    Table,
    Id,
    ClassId,
    Title,
    Description,
    MaxScore,
    Deadline,
    AllowLate,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Submissions {
    #[sea_orm(iden = "submissions")]
    Table,
    Id,
    HomeworkId,
    CreatorId,
    Version,
    Content,
    Status,
    IsLate,
    SubmittedAt,
}

#[derive(DeriveIden)]
enum Grades {
    #[sea_orm(iden = "grades")]
    Table,
    Id,
    SubmissionId,
    GraderId,
    Score,
    Comment,
    GradedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Files {
    #[sea_orm(iden = "files")]
    Table,
    Id,
    UserId,
    OriginalName,
    StoredName,
    FileType,
    FileSize,
    FilePath,
    DownloadToken,
    CitationCount,
    CreatedAt,
}

#[derive(DeriveIden)]
enum HomeworkFiles {
    #[sea_orm(iden = "homework_files")]
    Table,
    HomeworkId,
    FileId,
}

#[derive(DeriveIden)]
enum SubmissionFiles {
    #[sea_orm(iden = "submission_files")]
    Table,
    SubmissionId,
    FileId,
}

#[derive(DeriveIden)]
enum Notifications {
    #[sea_orm(iden = "notifications")]
    Table,
    Id,
    UserId,
    Type,
    Title,
    Content,
    ReferenceType,
    ReferenceId,
    IsRead,
    CreatedAt,
}
