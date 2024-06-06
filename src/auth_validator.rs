use crate::{
    database::get_collection,
    models::{AppState, Claims, User},
};
use actix_web::{dev::ServiceRequest, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{
    decode, encode, errors::ErrorKind, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use mongodb::bson::doc;

pub(crate) fn encode_user(user: &User) -> Result<String, jsonwebtoken::errors::Error> {
    let key = env!("jwt_key");
    let claims: Claims = user.into();
    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(key.as_bytes()),
    );
    token
}

pub(crate) async fn ok_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    let key = env!("jwt_key");
    let token = credentials.token();
    let app_data: &web::Data<AppState> = req
        .app_data()
        .expect("error getting app data from middleware");

    let users: mongodb::Collection<User> = get_collection(
        &app_data.connection_pool,
        &app_data.database_name,
        &app_data.user_collection_name,
    );

    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_required_spec_claims(&["exp", "sub"]);

    let token_data = match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(key.as_bytes()),
        &validation,
    ) {
        Ok(c) => c,
        Err(err) => match *err.kind() {
            ErrorKind::InvalidToken => panic!("Token is invalid"), // Example on how to handle a specific error
            ErrorKind::InvalidIssuer => panic!("Issuer is invalid"), // Example on how to handle a specific error
            _ => panic!("Some other errors"),
        },
    };
    let received_claims = token_data.claims;
    let user_id = received_claims.sub;
    match users
        .find_one(
            doc! {
                "_id": user_id
            },
            None,
        )
        .await
        .expect("user not found")
    {
        Some(_) => Ok(req),
        None => Err((
            actix_web::error::ErrorUnauthorized("no associated user found for this id"),
            req,
        )),
    }
}
