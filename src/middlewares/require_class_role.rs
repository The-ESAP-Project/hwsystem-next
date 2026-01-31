/*!
 * 基于班级角色的访问控制中间件
 *
 * 此中间件必须在 RequireJWT 中间件之后使用，用于验证用户是否具有特定班级角色权限。
 *
 * ## 使用方法
 *
 * ```rust,ignore
 * use actix_web::{web, App, HttpServer};
 * use crate::middlewares::require_jwt::RequireJWT;
 * use crate::middlewares::require_role::RequireClassRole;
 * use crate::models::class_users::entities::ClassUserRole;
 *
 * HttpServer::new(|| {
 *     App::new()
 *         .service(
 *             web::scope("/api")
 *                 .wrap(RequireJWT)  // 先验证JWT
 *                 .service(
 *                     web::scope("/classes")
 *                         .wrap(RequireClassRole::new("admin"))  // 再验证班级角色
 *                         .route("/students", web::get().to(list_students_handler))
 *                 )
 *         )
 * })
 * ```
 *
 * 或者验证多个班级角色：
 *
 * ```rust,ignore
 * .wrap(RequireClassRole::new_any(&["admin", "moderator"]))  // 任一班级角色即可
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
use std::{rc::Rc, sync::Arc};

use crate::{
    cache::{CacheResult, ObjectCache},
    models::{
        ErrorCode,
        class_users::entities::{ClassUser, ClassUserRole},
        users::entities::{User, UserRole},
    },
    storage::Storage,
};

use super::create_error_response;

#[derive(Clone)]
pub struct RequireClassRole {
    required_roles: Vec<ClassUserRole>,
    require_all: bool, // true表示需要所有班级角色，false表示任一班级角色即可
}

impl RequireClassRole {
    /// 创建需要特定班级角色的中间件
    pub fn new(role: &ClassUserRole) -> Self {
        Self {
            required_roles: vec![role.clone()],
            require_all: true,
        }
    }

    /// 创建需要任一班级角色的中间件
    pub fn new_any(roles: &[&ClassUserRole]) -> Self {
        Self {
            required_roles: roles.iter().map(|r| (*r).clone()).collect(),
            require_all: false,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequireClassRole
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = RequireClassRoleMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequireClassRoleMiddleware {
            service: Rc::new(service),
            required_roles: self.required_roles.clone(),
            require_all: self.require_all,
        }))
    }
}

pub struct RequireClassRoleMiddleware<S> {
    service: Rc<S>,
    required_roles: Vec<ClassUserRole>,
    require_all: bool,
}

impl<S, B> Service<ServiceRequest> for RequireClassRoleMiddleware<S>
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
            // 1. 校验用户信息
            let user_claims_opt = req.extensions().get::<User>().cloned();
            let user_claims = match user_claims_opt {
                Some(claims) => claims,
                None => {
                    return Ok(req.into_response(
                        create_error_response(
                            StatusCode::UNAUTHORIZED,
                            ErrorCode::Unauthorized,
                            "Unauthorized: missing user claims",
                        )
                        .map_into_right_body(),
                    ));
                }
            };

            // 2. 校验 class_id
            let class_id = match req
                .match_info()
                .get("class_id")
                .and_then(|s| s.parse::<i64>().ok())
            {
                Some(cid) => cid,
                None => {
                    return Ok(req.into_response(
                        create_error_response(
                            StatusCode::BAD_REQUEST,
                            ErrorCode::BadRequest,
                            "Missing or invalid class_id",
                        )
                        .map_into_right_body(),
                    ));
                }
            };

            // 3. 管理员直接放行
            if user_claims.role == UserRole::Admin {
                return Ok(srv.call(req).await?.map_into_left_body());
            }

            // 4. 查询用户在班级中的成员关系和角色
            let class_user = match get_class_user_by_user_id_and_class_id(
                &req,
                user_claims.id,
                class_id,
            )
            .await
            {
                Some(cs) => cs,
                None => {
                    return Ok(req.into_response(
                        create_error_response(
                            StatusCode::FORBIDDEN,
                            ErrorCode::ClassPermissionDenied,
                            "No permission for this class",
                        )
                        .map_into_right_body(),
                    ));
                }
            };

            // 5. 判断是否拥有所需角色
            let has_permission = if require_all {
                required_roles.iter().all(|role| &class_user.role == role)
            } else {
                required_roles.iter().any(|role| &class_user.role == role)
            };

            if has_permission {
                // 权限通过，插入 class_user 到扩展，继续后续处理
                tracing::debug!("Class user {} has permission", class_user.user_id);
                req.extensions_mut().insert(class_user);
                let res = srv.call(req).await?.map_into_left_body();
                Ok(res)
            } else {
                Ok(req.into_response(
                    create_error_response(
                        StatusCode::FORBIDDEN,
                        ErrorCode::ClassPermissionDenied,
                        "Access denied for this class role",
                    )
                    .map_into_right_body(),
                ))
            }
        })
    }
}

// 辅助函数：从请求中提取用户信息
impl RequireClassRole {
    /// 从请求扩展中提取用户 Class_User 信息
    /// 此函数应该在应用了RequireClassRole中间件的路由处理程序中使用
    pub fn extract_user_class_user(req: &actix_web::HttpRequest) -> Option<ClassUser> {
        req.extensions().get::<ClassUser>().cloned()
    }
}

/// 班级用户缓存 TTL（秒）
const CLASS_USER_CACHE_TTL: u64 = 300;

/// 生成班级用户缓存键
pub fn class_user_cache_key(user_id: i64, class_id: i64) -> String {
    format!("class_user:{}:{}", user_id, class_id)
}

async fn get_class_user_by_user_id_and_class_id(
    req: &ServiceRequest,
    user_id: i64,
    class_id: i64,
) -> Option<ClassUser> {
    let cache = req
        .app_data::<actix_web::web::Data<Arc<dyn ObjectCache>>>()
        .map(|data| data.get_ref().clone());

    let cache_key = class_user_cache_key(user_id, class_id);

    // 先查缓存
    if let Some(ref cache) = cache {
        if let CacheResult::Found(json) = cache.get_raw(&cache_key).await {
            if let Ok(class_user) = serde_json::from_str::<ClassUser>(&json) {
                tracing::debug!("Class user cache hit: {}", cache_key);
                return Some(class_user);
            }
        }
    }

    // 缓存未命中，查数据库
    let storage = req
        .app_data::<actix_web::web::Data<Arc<dyn Storage>>>()
        .map(|data| data.get_ref().clone())?;

    let class_user = match storage
        .get_class_user_by_user_id_and_class_id(user_id, class_id)
        .await
    {
        Ok(Some(class_user)) => class_user,
        Ok(None) => return None,
        Err(_) => return None,
    };

    // 存入缓存
    if let Some(cache) = cache {
        if let Ok(json) = serde_json::to_string(&class_user) {
            cache
                .insert_raw(cache_key, json, CLASS_USER_CACHE_TTL)
                .await;
        }
    }

    Some(class_user)
}
