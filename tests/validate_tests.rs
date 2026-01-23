//! 验证模块单元测试

use rust_hwsystem_next::utils::validate::{
    validate_email, validate_password, validate_password_simple, validate_username,
};

mod username_tests {
    use super::*;

    #[test]
    fn test_valid_usernames() {
        assert!(validate_username("hello").is_ok());
        assert!(validate_username("user123").is_ok());
        assert!(validate_username("test_user").is_ok());
        assert!(validate_username("test-user").is_ok());
        assert!(validate_username("User_Name-123").is_ok());
        assert!(validate_username("abcde").is_ok()); // 最小长度
        assert!(validate_username("1234567890123456").is_ok()); // 最大长度
    }

    #[test]
    fn test_username_too_short() {
        assert!(validate_username("abc").is_err());
        assert!(validate_username("ab").is_err());
        assert!(validate_username("a").is_err());
        assert!(validate_username("").is_err());
    }

    #[test]
    fn test_username_too_long() {
        assert!(validate_username("12345678901234567").is_err()); // 17 chars
        assert!(validate_username("abcdefghijklmnopqrstuvwxyz").is_err());
    }

    #[test]
    fn test_username_invalid_chars() {
        assert!(validate_username("user@name").is_err());
        assert!(validate_username("user name").is_err());
        assert!(validate_username("user.name").is_err());
        assert!(validate_username("user!name").is_err());
    }
}

mod email_tests {
    use super::*;

    #[test]
    fn test_valid_emails() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("user.name@domain.org").is_ok());
        assert!(validate_email("user+tag@example.co.uk").is_ok());
        assert!(validate_email("test123@test-domain.com").is_ok());
    }

    #[test]
    fn test_invalid_emails() {
        assert!(validate_email("invalid").is_err());
        assert!(validate_email("invalid@").is_err());
        assert!(validate_email("@domain.com").is_err());
        assert!(validate_email("test@domain").is_err());
        assert!(validate_email("test domain@example.com").is_err());
        assert!(validate_email("").is_err());
    }
}

mod password_tests {
    use super::*;

    #[test]
    fn test_valid_passwords() {
        assert!(validate_password("SecurePass1").is_valid);
        assert!(validate_password("MyP@ssw0rd").is_valid);
        assert!(validate_password("TestPass123").is_valid);
        assert!(validate_password("Aa1bcdefg").is_valid); // 最小有效密码（9 chars）
    }

    #[test]
    fn test_password_too_short() {
        let result = validate_password("Ab1");
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .contains(&"Password must be at least 8 characters long")
        );
    }

    #[test]
    fn test_password_no_uppercase() {
        let result = validate_password("abcd1234");
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .contains(&"Password must contain at least one uppercase letter")
        );
    }

    #[test]
    fn test_password_no_lowercase() {
        let result = validate_password("ABCD1234");
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .contains(&"Password must contain at least one lowercase letter")
        );
    }

    #[test]
    fn test_password_no_digit() {
        let result = validate_password("AbcdEfgh");
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .contains(&"Password must contain at least one digit")
        );
    }

    #[test]
    fn test_common_weak_passwords() {
        // 这些都是常见弱密码，即使符合其他规则也应该被拒绝
        let weak_passwords = ["password", "12345678", "Password1", "Qwerty123", "Abcd1234"];

        for pwd in weak_passwords {
            let result = validate_password(pwd);
            assert!(
                !result.is_valid || !result.errors.is_empty(),
                "Password '{}' should be rejected",
                pwd
            );
        }
    }

    #[test]
    fn test_password_simple_ok() {
        assert!(validate_password_simple("SecurePass123").is_ok());
    }

    #[test]
    fn test_password_simple_err() {
        let result = validate_password_simple("weak");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("8 characters"));
    }

    #[test]
    fn test_password_error_message() {
        let result = validate_password("abc");
        assert!(!result.is_valid);
        let msg = result.error_message();
        assert!(msg.contains("8 characters"));
        assert!(msg.contains("uppercase"));
    }
}
