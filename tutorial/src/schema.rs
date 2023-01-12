use microdb::{Database, DatabaseFactory, table::Table};
use microdb_derive::{Database, DatabaseFactory};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct BloggerStatistics
{
    pub post_count: usize,
    pub like_count: usize
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Blogger
{    
    pub name: String,
    pub statistics: BloggerStatistics
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Post
{
    pub user_id: usize,
    pub text: String
}

#[derive(Database, DatabaseFactory)]
pub struct BlogDatabase
{
    pub bloggers: Table::<Blogger>,
    pub posts: Table::<Post>
}