pub fn validate_username(username: &str) -> Result<(), &'static str> {
    // 用户名长度校验：5 <= x <= 16
    if username.len() < 5 || username.len() > 16 {
        return Err("Username length must be between 5 and 16 characters");
    }
    // 用户名格式校验：只能包含字母、数字、下划线或连字符
    let username_re = regex::Regex::new(r"^[A-Za-z0-9_-]+$").unwrap();
    if !username_re.is_match(username) {
        return Err("Username must contain only letters, numbers, underscores or hyphens");
    }
    Ok(())
}

pub fn validate_email(email: &str) -> Result<(), &'static str> {
    // 邮箱格式校验：必须包含 @ 和 .
    let email_re = regex::Regex::new(r"^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}$").unwrap();
    if !email_re.is_match(email) {
        return Err("Email format is invalid");
    }
    Ok(())
}

/// 密码策略验证结果
#[derive(Debug, Clone)]
pub struct PasswordValidationResult {
    pub is_valid: bool,
    pub errors: Vec<&'static str>,
}

impl PasswordValidationResult {
    pub fn error_message(&self) -> String {
        self.errors.join("; ")
    }
}

/// 验证密码是否符合安全策略
///
/// 策略要求：
/// - 最小长度：8 字符
/// - 必须包含：大写字母 + 小写字母 + 数字
/// - 可选：特殊字符（增强安全性）
pub fn validate_password(password: &str) -> PasswordValidationResult {
    let mut errors = Vec::new();

    // 1. 长度检查：至少 8 个字符
    if password.len() < 8 {
        errors.push("Password must be at least 8 characters long");
    }

    // 2. 大写字母检查
    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        errors.push("Password must contain at least one uppercase letter");
    }

    // 3. 小写字母检查
    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        errors.push("Password must contain at least one lowercase letter");
    }

    // 4. 数字检查
    if !password.chars().any(|c| c.is_ascii_digit()) {
        errors.push("Password must contain at least one digit");
    }

    // 5. 常见弱密码检查
    let weak_passwords = [
        "password",
        "12345678",
        "123456789",
        "qwerty123",
        "admin123",
        "password1",
        "Password1",
        "Qwerty123",
        "Abcd1234",
    ];
    if weak_passwords
        .iter()
        .any(|&weak| password.eq_ignore_ascii_case(weak))
    {
        errors.push("Password is too common, please choose a stronger password");
    }

    PasswordValidationResult {
        is_valid: errors.is_empty(),
        errors,
    }
}

/// 简化的密码验证（返回 Result）
pub fn validate_password_simple(password: &str) -> Result<(), String> {
    let result = validate_password(password);
    if result.is_valid {
        Ok(())
    } else {
        Err(result.error_message())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_password() {
        assert!(validate_password("SecureP@ss1").is_valid);
        assert!(validate_password("MyP@ssw0rd").is_valid);
        assert!(validate_password("SecurePass123").is_valid);
    }

    #[test]
    fn test_short_password() {
        let result = validate_password("Ab1");
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .contains(&"Password must be at least 8 characters long")
        );
    }

    #[test]
    fn test_no_uppercase() {
        let result = validate_password("abcd1234");
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .contains(&"Password must contain at least one uppercase letter")
        );
    }

    #[test]
    fn test_no_lowercase() {
        let result = validate_password("ABCD1234");
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .contains(&"Password must contain at least one lowercase letter")
        );
    }

    #[test]
    fn test_no_digit() {
        let result = validate_password("AbcdEfgh");
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .contains(&"Password must contain at least one digit")
        );
    }

    #[test]
    fn test_common_password() {
        let result = validate_password("Password1");
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .contains(&"Password is too common, please choose a stronger password")
        );
    }
}
