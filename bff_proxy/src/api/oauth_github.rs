use std::fmt::format;

use actix_web::{get, Responder, HttpResponse, http::header, web};
use actix_session::Session;
use reqwest::{StatusCode, Error};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
struct AccessCode {
    code: String,
    state: String,
}

#[derive(Deserialize)]
struct GitHubAuthReponse {
    access_token: String,
    token_type: String,
    scope: String,
}

#[derive(Deserialize, Serialize)]
struct GitHubUserResponse {
    login: String,
    id: u64,
    node_id: String,
    avatar_url: String,
    gravatar_id: Option<String>,
    url: String,
    html_url: String,
    followers_url: String,
    following_url: String,
    gists_url: String,
    starred_url: String,
    subscriptions_url: String,
    organizations_url: String,
    repos_url: String,
    events_url: String,
    received_events_url: String,
    type_: Option<String>,
    site_admin: bool,
    name: String,
    company: Option<String>,
    blog: Option<String>,
    location: Option<String>,
    email: Option<String>,
    hireable: bool,
    bio: Option<String>,
    twitter_username: Option<String>,
    public_repos: u64,
    public_gists: u64,
    followers: u64,
    following: u64,
    created_at: String,
    updated_at: String,
}

#[get("/signin/github")]
async fn login(session: Session) -> impl Responder {
    // redirect to the github oauth page
    let client_id = match std::env::var("GH_CLIENT_ID") {
        Ok(client_id) => client_id,
        Err(_) => return HttpResponse::InternalServerError().body("Error: CLIENT_ID not set!"),
    };

    let redirect_uri = match std::env::var("GH_REDIRECT_URI") {
        Ok(redirect_uri) => redirect_uri,
        Err(_) => return HttpResponse::InternalServerError().body("Error: REDIRECT_URI not set!"),
    };

    // Generate a random state to prevent CSRF attacks
    let state = Uuid::new_v4().to_string();
    

    let url = format!("https://github.com/login/oauth/authorize?scope=user:email&client_id={}&redirect_uri={}&state={}", client_id, redirect_uri, state);
    
    // Add the state to the session so we can grab it later and prevent CSRF attacks
    match session.insert("state", state) {
        Ok(_) => (),
        Err(e) => {
            println!("Error: {}", e);
            return HttpResponse::InternalServerError().body("Error: Could not set session state!");
        }
    }

    // Redirect to the github oauth page
    let mut response = HttpResponse::Ok();
    response.status(StatusCode::TEMPORARY_REDIRECT);
    response.append_header((
        header::LOCATION, 
        url
    ));

    return response.into();
}

#[get("/signin/github/callback")]
async fn callback(params: web::Query<AccessCode>, session: Session) -> impl Responder {
    // Gather the required information for exchanging the code for an access token
    let client_id = match std::env::var("GH_CLIENT_ID") {
        Ok(client_id) => client_id,
        Err(_) => return HttpResponse::InternalServerError().body("Error: CLIENT_ID not set!"),
    };

    let client_secret = match std::env::var("GH_CLIENT_SECRET") {
        Ok(client_secret) => client_secret,
        Err(_) => return HttpResponse::InternalServerError().body("Error: CLIENT_SECRET not set!"),
    };

    let redirect_uri = match std::env::var("GH_REDIRECT_URI") {
        Ok(redirect_uri) => redirect_uri,
        Err(_) => return HttpResponse::InternalServerError().body("Error: REDIRECT_URI not set!"),
    };

    let state = match session.get::<String>("state") {
        Ok(state) => match state {
            Some(state) => state,
            None => return HttpResponse::InternalServerError().body("Error: Could not get session state!"),
        },
        Err(e) => {
            println!("Error: {}", e);
            return HttpResponse::InternalServerError().body("Error: Could not get session state!");
        }
    };

    let code = params.code.clone();

    if state != params.state {
        return HttpResponse::InternalServerError().body("Error: State does not match!");
    }

    // Exchange the code for an access token
    let url = format!("https://github.com/login/oauth/access_token?code={}&client_id={}&client_secret={}&redirect_uri={}", code, client_id, client_secret, redirect_uri);

    let client = reqwest::Client::new();
    let res = match client.post(url)
    .header("Accept", "application/json")
    .send()
    .await {
        Ok(res) => match res.json::<GitHubAuthReponse>().await {
            Ok(access_token) => access_token,
            Err(e) => {
                println!("Error: {}", e);
                return HttpResponse::InternalServerError().body("Error: Could not get access token!");
            }
        },
        Err(e) => {
            println!("Error: {}", e);
            return HttpResponse::InternalServerError().body("Error: Could not get access token!");
        }
    };

    // Clean up the session
    session.remove("state");
    match session.insert("access_token", res.access_token.clone()) {
        Ok(_) => (),
        Err(e) => {
            println!("Error: {}", e);
            return HttpResponse::InternalServerError().body("Error: Could not set session access token!");
        }
    };

    // 1. Get the user's info
    let url = "https://api.github.com/user";
    let data = match client
    .get(url)
    .header("X-GitHub-Api-Version", "2022-11-28")
    .header("Accept", "application/json")
    .header("User-Agent", "Caveman Learn App")
    .bearer_auth(res.access_token)
    .send()
    .await {
        Ok(res) => res,
        Err(e) => {
            println!("Error: {}", e);
            return HttpResponse::InternalServerError().body("Error: Could not fetch user info!");
        }
    };

    // let text = data.text().await.unwrap();

    let user = match data.json::<GitHubUserResponse>().await {
        Ok(user) => user,
        Err(e) => {
            println!("Error: {}", e);
            return HttpResponse::InternalServerError().body("Error: Could not deserialize user info!");
        }
    };

    // 2. record the user in the database as a new user if they don't exist
    
    // 3. redirect the user to the frontend

    HttpResponse::Ok().body(user.name)
}
