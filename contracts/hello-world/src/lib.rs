#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, Address, symbol_short};

// Structure to store blog post metadata
#[contracttype]
#[derive(Clone)]
pub struct BlogPost {
    pub post_id: u64,
    pub author: Address,
    pub ipfs_hash: String,      // IPFS hash where content is stored
    pub title: String,
    pub timestamp: u64,
    pub is_active: bool,         // for soft deletion
}

// Structure to track platform statistics
#[contracttype]
#[derive(Clone)]
pub struct BlogStats {
    pub total_posts: u64,
    pub active_posts: u64,
    pub total_authors: u64,
}

// Mapping post_id to BlogPost
#[contracttype]
pub enum PostBook {
    Post(u64)
}

// Mapping author address to their post count
#[contracttype]
pub enum AuthorBook {
    Author(Address)
}

// Counter for generating unique post IDs
const POST_COUNT: Symbol = symbol_short!("P_COUNT");

// Key for storing blog statistics
const STATS: Symbol = symbol_short!("STATS");

#[contract]
pub struct DecentralizedBlogContract;

#[contractimpl]
impl DecentralizedBlogContract {
    
    // Function to create a new blog post
    pub fn create_post(env: Env, author: Address, ipfs_hash: String, title: String) -> u64 {
        // Verify the author
        author.require_auth();
        
        // Get and increment post counter
        let mut post_count: u64 = env.storage().instance().get(&POST_COUNT).unwrap_or(0);
        post_count += 1;
        
        // Get current timestamp
        let timestamp = env.ledger().timestamp();
        
        // Create new blog post
        let new_post = BlogPost {
            post_id: post_count,
            author: author.clone(),
            ipfs_hash,
            title,
            timestamp,
            is_active: true,
        };
        
        // Update statistics
        let mut stats = Self::get_stats(env.clone());
        stats.total_posts += 1;
        stats.active_posts += 1;
        
        // Check if author is new
        let author_post_count: u64 = env.storage()
            .instance()
            .get(&AuthorBook::Author(author.clone()))
            .unwrap_or(0);
        
        if author_post_count == 0 {
            stats.total_authors += 1;
        }
        
        // Update author's post count
        env.storage()
            .instance()
            .set(&AuthorBook::Author(author.clone()), &(author_post_count + 1));
        
        // Store the post
        env.storage().instance().set(&PostBook::Post(post_count), &new_post);
        
        // Store updated statistics
        env.storage().instance().set(&STATS, &stats);
        
        // Store updated post count
        env.storage().instance().set(&POST_COUNT, &post_count);
        
        // Extend storage TTL
        env.storage().instance().extend_ttl(10000, 10000);
        
        log!(&env, "Blog post created with ID: {}", post_count);
        post_count
    }
    
    // Function to deactivate a blog post (soft delete)
    pub fn deactivate_post(env: Env, post_id: u64, author: Address) {
        // Verify the author
        author.require_auth();
        
        // Get the post
        let mut post = Self::get_post(env.clone(), post_id);
        
        // Verify the author owns this post and it's active
        if post.author != author {
            log!(&env, "Unauthorized: You are not the author of this post");
            panic!("Unauthorized access");
        }
        
        if !post.is_active {
            log!(&env, "Post is already deactivated");
            panic!("Post already deactivated");
        }
        
        // Deactivate the post
        post.is_active = false;
        
        // Update statistics
        let mut stats = Self::get_stats(env.clone());
        stats.active_posts -= 1;
        
        // Store updated post and statistics
        env.storage().instance().set(&PostBook::Post(post_id), &post);
        env.storage().instance().set(&STATS, &stats);
        
        env.storage().instance().extend_ttl(10000, 10000);
        
        log!(&env, "Blog post {} deactivated", post_id);
    }
    
    // Function to retrieve a blog post by ID
    pub fn get_post(env: Env, post_id: u64) -> BlogPost {
        let key = PostBook::Post(post_id);
        
        env.storage().instance().get(&key).unwrap_or(BlogPost {
            post_id: 0,
            author: Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")),
            ipfs_hash: String::from_str(&env, "Not_Found"),
            title: String::from_str(&env, "Not_Found"),
            timestamp: 0,
            is_active: false,
        })
    }
    
    // Function to get platform statistics
    pub fn get_stats(env: Env) -> BlogStats {
        env.storage().instance().get(&STATS).unwrap_or(BlogStats {
            total_posts: 0,
            active_posts: 0,
            total_authors: 0,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    
    #[test]
    fn test_create_post() {
        let env = Env::default();
        let contract_id = env.register_contract(None, DecentralizedBlogContract);
        let client = DecentralizedBlogContractClient::new(&env, &contract_id);
        
        let author = Address::generate(&env);
        let ipfs_hash = String::from_str(&env, "QmXxx123...");
        let title = String::from_str(&env, "My First Blog Post");
        
        env.mock_all_auths();
        
        let post_id = client.create_post(&author, &ipfs_hash, &title);
        
        assert_eq!(post_id, 1);
        
        let post = client.get_post(&post_id);
        assert_eq!(post.author, author);
        assert_eq!(post.is_active, true);
    }
}
