/*!
 * 基于角色的访问控制中间件
 *
 * 此中间件必须在 RequireJWT 中间件之后使用，用于验证用户是否具有特定角色权限。
 *
 * ## 使用方法
 *
 * ```rust,ignore
 * use actix_web::{web, App, HttpServer};
 * use crate::middlewares::require_jwt::RequireJWT;
 * use crate::middlewares::require_role::RequireRole;
 * use crate::models::users::entities::UserRole;
 *
 * HttpServer::new(|| {
 *     App::new()
 *         .service(
 *             web::scope("/api")
 *                 .wrap(RequireJWT)  // 先验证JWT
 *                 .service(
 *                     web::scope("/admin")
 *                         .wrap(RequireRole::new(&UserRole::Admin)))  // 再验证角色
 *                         .route("/users", web::get().to(admin_users_handler))
 *                 )
 *         )
 * })
 * ```
 *
 * 或者验证多个角色：
 *
 * ```rust,ignore
 * .wrap(RequireRole::new_any(UserRole::admin_roles()))  // 任一角色即可
 * ```
 */

use actix_service::{Service, Transform};
use actix_web::{
    Error, HttpMessage,
    body::EitherBody,
    dev::{ServiceRequest, ServiceResponse},
    http::StatusCode,
};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use std::rc::Rc;
use tracing::info;

use crate::{
    middlewares::RequireJWT,
    models::{
        ErrorCode,
        users::entities::{self, UserRole},
    },
};

use super::create_error_response;

#[derive(Clone)]
pub struct RequireRole {
    required_roles: Vec<UserRole>,
    require_all: bool, // true表示需要所有角色，false表示任一角色即可
}

impl RequireRole {
    /// 创建需要特定角色的中间件
    pub fn new(role: &UserRole) -> Self {
        Self {
            required_roles: vec![role.clone()],
            require_all: true,
        }
    }

    /// 创建需要任一角色的中间件
    pub fn new_any(roles: &[&UserRole]) -> Self {
        Self {
            required_roles: roles.iter().map(|r| (*r).clone()).collect(),
            require_all: false,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequireRole
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = RequireRoleMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequireRoleMiddleware {
            service: Rc::new(service),
            required_roles: self.required_roles.clone(),
            require_all: self.require_all,
        }))
    }
}

pub struct RequireRoleMiddleware<S> {
    service: Rc<S>,
    required_roles: Vec<UserRole>,
    require_all: bool,
}

impl<S, B> Service<ServiceRequest> for RequireRoleMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();
        let required_roles = self.required_roles.clone();
        let require_all = self.require_all;

        Box::pin(async move {
            // 从请求扩展中获取用户Claims
            let user_claims = req.extensions().get::<entities::User>().cloned();

            match user_claims {
                Some(claims) => {
                    let user_sub = claims.id;
                    let user_role = RequireJWT::extract_user_role(req.request());
                    let has_permission = if require_all {
                        // 需要所有角色（通常用于单一角色验证）
                        required_roles
                            .iter()
                            .all(|role| user_role.as_ref() == Some(role))
                    } else {
                        // 需要任一角色
                        required_roles
                            .iter()
                            .any(|role| user_role.as_ref() == Some(role))
                    };

                    if has_permission {
                        let res = srv.call(req).await?.map_into_left_body();
                        Ok(res)
                    } else {
                        info!(
                            "Access denied for user {} (role: {:?}). Required roles: {:?}",
                            user_sub, user_role, required_roles
                        );
                        Ok(req.into_response(
                            create_error_response(
                                StatusCode::FORBIDDEN,
                                ErrorCode::Forbidden,
                                "Access denied.",
                            )
                            .map_into_right_body(),
                        ))
                    }
                }
                None => {
                    info!(
                        "Role check failed: No user claims found in request. Make sure RequireJWT middleware is applied first."
                    );
                    Ok(req.into_response(
                        create_error_response(
                            StatusCode::UNAUTHORIZED,
                            ErrorCode::Unauthorized,
                            "Authentication required",
                        )
                        .map_into_right_body(),
                    ))
                }
            }
        })
    }
}
