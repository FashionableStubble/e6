use reqwest::header::{AUTHORIZATION, USER_AGENT};
// use indicatif::{ MultiProgress, ProgressBar, ProgressStyle };
use base64::{engine, prelude::*};
use serde::{Serialize, Deserialize};

#[derive(Debug, serde::Deserialize, Default, Clone)]
pub enum Rating {
    s,
    #[default]
    e,
    q,
}

impl ToString for Rating {
    fn to_string(&self) -> String {
        match self {
            Rating::e => "explicit",
            Rating::q => "questionable",
            Rating::s => "safe",
        }.to_string()
    }
}

#[derive(Debug, serde::Deserialize, Default, Clone)]
pub struct Relationships {
    pub has_children: bool,
    pub has_active_children: bool,
    pub children: Vec<u64>,
    pub parent_id: Option<u64>,
}


#[derive(Debug, serde::Deserialize, Default, Clone)]
pub struct Tags {
    #[serde(rename = "artist")]
    pub artists: Vec<String>,
    pub character: Vec<String>,
    pub contributor: Vec<String>,
    pub copyright: Vec<String>,
    pub general: Vec<String>,
    pub invalid: Vec<String>,
    pub lore: Vec<String>,
    pub meta: Vec<String>,
    pub species: Vec<String>,
}

#[derive(Debug, serde::Deserialize, Default, Clone)]
pub struct Score {
    pub up: u32,
    pub down: i32, // since e6 is weird and counts downvotes negatively
    pub total: i32
}

#[derive(Debug, serde::Deserialize, Default, Clone, Copy)]
pub enum FileExt {
    png,
    mp4,
    webm,
    jpg,
    webp,
    swf,
    gif,
    #[default]
    unk
}

impl ToString for FileExt {
    fn to_string(&self) -> String {
        use FileExt::*;

        match self {
            gif => "gif",
            mp4 => "mp4",
            webp => "webp",
            png => "png",
            webm => "webm",
            swf => "swf",
            jpg => "jpg",
            unk => "data",
        }.to_string()
    }
}

#[derive(Debug, serde::Deserialize, Default, Clone)]
pub struct FileEntry {
    pub ext: FileExt,
    pub url: Option<String>,
    pub size: u64,
    pub md5: Option<String>,
    pub height: Option<u32>,
    pub width: Option<u32>
}

#[derive(Debug, serde::Deserialize, Default, Clone)]
pub struct PreviewEntry {
    pub url: Option<String>,
    pub height: Option<u32>,
    pub width: Option<u32>,
    pub alt: Option<String>
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Flags {
    pub pending: bool,
    pub flagged: bool,
    pub note_locked: bool,
    pub status_locked: bool,
    pub rating_locked: bool,
    pub deleted: bool
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Alternative {
    pub fps: Option<f32>,
    pub codec: Option<String>,
    pub size: Option<u64>,
    pub width: u16,
    pub height: u16,
    pub url: Option<String>
}

// adding them as I run into them
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Variants {
    pub mp4: Option<Alternative>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Alternatives {
    pub has: bool,
    pub original: Option<Alternative>,
    pub variants: Option<Variants>,
    pub samples: Option<Samples>
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SampleEntry {
    pub has: bool,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub url: Option<String>,
    pub alt: Option<String>,
    pub alternatives: Option<Alternatives>
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Samples {
    #[serde(rename = "480p")]
    pub sample_480p: Alternative
}

#[derive(Debug, serde::Deserialize, Default, Clone)]
pub struct Post {
    pub file: FileEntry,
    pub tags: Tags,
    pub id: u64,
    pub pools: Vec<u64>,
    pub rating: Rating,
    pub relationships: Relationships,
    pub description: String,
    pub created_at: String,
    pub comment_count: u32,
    pub uploader_id: u32,
    pub updated_at: Option<String>,
    pub fav_count: u32,
    pub sources: Option<Vec<String>>,
    pub score: Score,
    pub preview: PreviewEntry,
    pub locked_tags: Option<Vec<String>>,
    pub change_seq: u64,
    pub flags: Flags,
    pub approver_id: Option<u32>,
    pub uploader_name: String,
    pub is_favorited: bool,
    pub has_notes: bool,
    pub duration: Option<f32>,
    pub sample: Option<Alternative>
}

#[derive(Debug, Deserialize, Default)]
pub struct Posts {
    posts: Vec<Post>
}

impl From<Vec<Post>> for Posts {
    fn from(value: Vec<Post>) -> Self {
        Posts { posts: value }
    }
}

impl IntoIterator for Posts {
    type Item = Post;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.posts.into_iter()
    }
}

pub struct E6 {
    client: reqwest::Client,
    auth: String,
    user_agent: String
}

impl E6 {
    pub fn new(key: &str, app_name: &str, app_version: &str, username: &str) -> Self {
        E6 {
            auth: engine::general_purpose::STANDARD.encode(format!("{username}:{key}")),
            client: reqwest::Client::new(),
            user_agent: format!("{app_name}/{app_version} (by {username})")
        }
        
    }

    pub async fn fetch_posts(&self, tags: &Vec<String>, index: Paginate) -> Vec<Post> {
        self.client
            .get(format!("https://e621.net/posts.json?tags={}&limit=320&page={}", tags.join("+"), index.to_param()))
            .header(USER_AGENT, &self.user_agent)
            .header(AUTHORIZATION, &self.auth)
            .send()
            .await.unwrap()
            .json::<Posts>()
            .await.unwrap()
            .posts
    }

    pub async fn search(&self, tags: Vec<String>) -> Posts {
        let mut posts = self.fetch_posts(&tags, 1.into()).await;

        let mut post_list = posts.clone();
        
        let Some(previous_last_post) = posts.last() else {
            eprintln!("No post found for the tags: \"{}\"", tags.join(" "));
            return Posts::default();
        };

        let mut previous_last_id = previous_last_post.id;

        // let main_progress_bar = ProgressBar::no_length().with_style(ProgressStyle::default_bar().template("[Processing Pages] {spinner:.green} [{elapsed_precise}] [{bar:40.green/cyan}] {pos:>7}").unwrap().progress_chars("#>-"));
        // let multi_progress_handler = MultiProgress::new();
        
        // let main_progress_bar = multi_progress_handler.add(main_progress_bar);
        
        loop {
            posts = self.fetch_posts(&tags, Paginate::ID(previous_last_id)).await;

            if posts.len() == 0 || posts[posts.len() - 1].id == previous_last_id {
                #[cfg(debug_assertions)]
                println!("Reached last page, wrapping up... Current: {}, Previous: {previous_last_id}", match posts.get(0) { Some(post) => post.id.to_string(), None => "No Current Post".into() });
                break;
            } else {
                previous_last_id = posts[posts.len() - 1].id.to_owned();
            }

            // main_progress_bar.inc(1);
            post_list.extend(posts);
        }

        // main_progress_bar.finish();

        Posts {
            posts: post_list
        }
    }
}

pub enum Paginate {
    Page(u64),
    ID(u64)
}

impl Paginate {
    fn to_param(&self) -> String {
        match self {
            Paginate::Page(p) => p.to_string(),
            Paginate::ID(id) => format!("b{id}")
        }
    }
}

impl From<u64> for Paginate {
    fn from(value: u64) -> Self {
        Paginate::Page(value)
    }
}
