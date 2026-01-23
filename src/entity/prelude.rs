//! 预导入模块，方便使用

pub use super::class_users::{
    ActiveModel as ClassUserActiveModel, Entity as ClassUsers, Model as ClassUserModel,
};
pub use super::classes::{ActiveModel as ClassActiveModel, Entity as Classes, Model as ClassModel};
pub use super::files::{ActiveModel as FileActiveModel, Entity as Files, Model as FileModel};
pub use super::grades::{ActiveModel as GradeActiveModel, Entity as Grades, Model as GradeModel};
pub use super::homework_files::{
    ActiveModel as HomeworkFileActiveModel, Entity as HomeworkFiles, Model as HomeworkFileModel,
};
pub use super::homeworks::{
    ActiveModel as HomeworkActiveModel, Entity as Homeworks, Model as HomeworkModel,
};
pub use super::notifications::{
    ActiveModel as NotificationActiveModel, Entity as Notifications, Model as NotificationModel,
};
pub use super::submission_files::{
    ActiveModel as SubmissionFileActiveModel, Entity as SubmissionFiles,
    Model as SubmissionFileModel,
};
pub use super::submissions::{
    ActiveModel as SubmissionActiveModel, Entity as Submissions, Model as SubmissionModel,
};
pub use super::users::{ActiveModel as UserActiveModel, Entity as Users, Model as UserModel};
