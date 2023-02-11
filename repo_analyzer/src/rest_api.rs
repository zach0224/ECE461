use std::collections::HashMap;
use std::{env, time};
use std::str;
use std::env::VarError;
use std::error::{Error};
use std::io::ErrorKind;
use std::result::{Result};
use reqwest::{Client, get, Response, StatusCode};
use reqwest::header::HeaderMap;
use base64::{Engine, engine::general_purpose};
use serde_json::Value;
use async_recursion::async_recursion;


///
/// Returns the github link associated with the npmjs package
///
/// # Arguments
///
/// * 'url' - The full npmjs url you want the associated github link of
///
pub async fn npmjs_get_repository_link(repository: &str) -> Result<String, Box<dyn Error>> {
    // docs of the api to call to get the github link
    // https://api-docs.npms.io/#api-Package


    let request_url_str = format!("https://api.npms.io/v2/package/{}", repository);

    let client = Client::new();
    let res = client
        .get(request_url_str)
        .send().await?;
    // let status_code = res.status();
    // println!("status code: {}", status_code);

    let result_text_res = res.text().await;
    if result_text_res.is_err() {
        return Err(Box::try_from(result_text_res.unwrap_err()).unwrap());
    }

    let result_text = result_text_res.unwrap().to_owned();

    let json_obj_res = serde_json::from_str(&result_text);
    if json_obj_res.is_err() {
        return Err(Box::try_from(json_obj_res.unwrap_err()).unwrap());
    }

    let json_obj: serde_json::Value = json_obj_res.unwrap();
    // println!("{:#?}", json_obj);
    let json_collected_res = json_obj.get("collected");
    if json_collected_res.is_none() {
        return Err(panic!("Failed to get repository link from npmjs package{}", repository));
    }
    let json_metadata_res = json_collected_res.unwrap().get("metadata");
    if json_metadata_res.is_none() {
        return Err(panic!("Failed to get repository link from npmjs package{}", repository));
    }
    let json_links_res = json_metadata_res.unwrap().get("links");
    if json_links_res.is_none() {
        return Err(panic!("Failed to get repository link from npmjs package{}", repository));
    }
    let json_repository_res = json_links_res.unwrap().get("repository");
    if json_repository_res.is_none() {
        return Err(panic!("Failed to get repository link from npmjs package{}", repository));
    }
    let repo_link_res = json_repository_res.unwrap().as_str();
    if repo_link_res.is_none() {
        return Err(panic!("Failed to get repository link from npmjs package{}", repository));
    }
    let repo_link_str = repo_link_res.unwrap();


    // let t = json_obj.get("collected").unwrap()
    //                         .get("metadata").unwrap()
    //                         .get("links").unwrap()
    //                         .get("repository").unwrap()
    //                         .as_str().unwrap();

    // println!("{}", t);
    Ok(repo_link_str.to_owned())
}


///
/// # Info
/// Returns a HashMap<String, String> with the following keys:
///
/// key
///
/// documentation_length:       The length of the documentation in lines (excluding blank lines).
///
/// codebase_length:            The length of the codebase in lines (excluding blank lines).
///
/// num_closed_bugs_month:      The number of closed bugs in the last month.
///
/// num_opened_issues_month:    The number of opened issues in the last month.
///
/// license:                    The name of the license the repository is using.
///
///
/// # Arguments
///
/// * owner: &str -             The username of the owner of the target repository
///
/// * repository: &str -        The name of the repository
///
pub async fn github_get_metrics(owner: &str, repository: &str) -> Result<HashMap<String, String>, Box<dyn Error>> {
// metrics:
// license
// codebase length
// README
// opened issues
// # commits
// # committers
    let mut metrics: HashMap<String, String> = HashMap::new();
    let issues_path = format!("{}/issues", repository);
    let commits_path = format!("{}/commits", repository);

    // get codebase length
    let response_res = github_get_response_body(owner, repository, None).await;
    if response_res.is_err() {
        return Err(response_res.err().unwrap())
    }
    let response = response_res.unwrap();

    let codebase_length_res = response.get("size");
    if codebase_length_res.is_none() {
        return Err(panic!("Failed to get codebase size of {}/{}", owner, repository));
    }
    let codebase_length_val = codebase_length_res.unwrap().as_i64();
    if codebase_length_val.is_none() {
        return Err(panic!("Failed to get codebase size of {}/{}", owner, repository));
    }
    metrics.insert(String::from("codebase_length"), format!("{}", codebase_length_val.unwrap()));

    // get # opened issues
    let open_issues_res = response.get("open_issues_count");
    if open_issues_res.is_none() {
        return Err(panic!("Failed to get number of open issues of {}/{}", owner, repository));
    }

    let open_issues_val = open_issues_res.unwrap().as_i64();
    if open_issues_val.is_none() {
        return Err(panic!("Failed to get number of open issues of {}/{}", owner, repository));
    }

    metrics.insert(String::from("open_issues"), format!("{}", open_issues_val.unwrap()));

    // get # commits
    let commits_response_res = github_get_response_body(owner, &commits_path, None).await;
    if commits_response_res.is_err() {
        return Err(commits_response_res.err().unwrap());
    }

    let commits_response = commits_response_res.unwrap();
    let commits_arr_res = commits_response.as_array();
    if commits_arr_res.is_none() {
        return Err(panic!("Failed to get number of commits of {}/{}", owner, repository));
    }
    let commits_arr = commits_arr_res.unwrap();
    // println!("{}", commits_arr.len());

    // get # committers

    // get license / README

    
    Ok(metrics)
}


///
/// Returns the response body as a serde_json::Value
///
/// # Arguments
///
pub async fn github_get_response_body(owner: &str, repository: &str, headers: Option<HeaderMap>) -> Result<serde_json::Value, Box<dyn Error>> {
    let response_res = github_get_response(owner, repository, headers).await;
    if response_res.is_err() {
        return Err(Box::try_from(response_res.err().unwrap()).unwrap());
    }
    let response = response_res.unwrap();
    let response_text_res = response.text().await;
    if response_text_res.is_err() {
        return Err(Box::try_from(response_text_res.err().unwrap()).unwrap())
    }

    let response_text = response_text_res.unwrap().to_owned();
    let response_json_res = serde_json::from_str(&response_text);
    if response_json_res.is_err() {
        return Err(Box::from(response_json_res.err().unwrap()))
    }
    let response_json = response_json_res.unwrap();

    Ok(response_json)
}


///
/// Returns the StatusCode of a request to a Github repository
///
/// # Arguments
///
/// * 'owner'      :&str - The username of the owner of the repository
/// * 'repository' :&str - The name of the repository
///
pub async fn github_get_status(owner: &str, repository: &str) -> Result<StatusCode, Box<dyn Error>> {
    let response_res = github_get_response(owner, repository, None).await;
    if response_res.is_err() {
        return Err(Box::try_from(response_res.err().unwrap()).unwrap());
    }
    Ok(response_res.unwrap().status().to_owned())
}


///
/// Returns the Response object from a github url
///
/// # Arguments
///
pub async fn github_get_response(owner: &str, repository: &str, headers: Option<HeaderMap>) -> Result<Response, Box<dyn Error>> {
    let mut owner_mut = String::from(owner);
    let mut repo_mut = String::from(repository);
    if !owner.is_empty() {
        owner_mut.insert(0, '/')
    }
    if !repository.is_empty() {
        repo_mut.insert(0, '/');
    }
    let url = format!("https://api.github.com/repos{}{}", owner_mut, repo_mut);
    let token_res = get_github_api_token();
    if token_res.is_err() {
        return Err(Box::try_from(token_res.err().unwrap()).unwrap());
    }
    let token = token_res.unwrap();

    let client_id = "";
    let client_secret = "";


    let client = Client::new();
    let mut request_builder = client
        .get(url)
        .header("Authorization", token)
        .header("client_id", client_id)
        .header("client_secret", client_secret)
        .header("User-Agent", "ECE461-repository-analyzer");
    if headers.is_some() {
        request_builder = request_builder.headers(headers.unwrap());
    }
    // std::thread::sleep(time::Duration::from_millis(10));
    let response_res = request_builder.send().await;
    if response_res.is_err() {
        return Err(Box::try_from(response_res.err().unwrap()).unwrap());
    }
    let response = response_res.unwrap();
    // println!("{:#?}", response);
    Ok(response)
}


//////////////////////////
////                  ////
//// helper functions ////
////                  ////
//////////////////////////


fn get_github_api_token() -> Result<String, VarError> {
    let name = "GITHUB_TOKEN";
    let res = env::var(name);
    if res.is_err() {
        println!("${} is not set in Enviromental Variables", name);
        return Err(res.err().unwrap())
    }
    Ok(res.unwrap())
}