pub(super) const CRUD_API_ROUTES: &str = r#"import { listBlogs, createBlog, updateBlog } from "@/server/handlers/blogs-handler"
import { registerUser, loginUser, getSession, logoutUser } from "@/server/handlers/users-handler"
import requireBearer from "@/server/middlewares/auth"

endpoints apiRoutes
  group path:"/api/auth"
    post path:"/register" handler:registerUser
    post path:"/login" handler:loginUser
    get path:"/session" handler:getSession middleware:[requireBearer]
    post path:"/logout" handler:logoutUser middleware:[requireBearer]
  group path:"/api/blogs"
    get path:"" handler:listBlogs
    post path:"" handler:createBlog middleware:[requireBearer]
    patch path:"/:id" handler:updateBlog middleware:[requireBearer]
"#;

pub(super) const CRUD_AUTH_TYPES: &str = r#"type RegisterInput
  name:string
  email:string
  password:string

type LoginInput
  email:string
  password:string
"#;

pub(super) const CRUD_BLOG_TYPES: &str = r#"type BlogInput
  title:string
  content:string

type BlogPatch
  title?:string
  content?:string
"#;

pub(super) const CRUD_USERS: &str = r#"entity Users
  id:string primary:true
  name:string required:true
  email:string required:true unique:true
  password:string required:true
  createdAt:timestamp required:true
"#;

pub(super) const CRUD_BLOGS: &str = r#"entity Blogs
  id:string primary:true
  title:string required:true
  content:string required:true
  ownerId:string required:true index:true
  createdAt:timestamp required:true
  updatedAt:timestamp required:true
"#;

pub(super) const CRUD_SESSIONS: &str = r#"entity Sessions
  id:string primary:true
  userId:string required:true index:true
  createdAt:timestamp required:true
"#;

pub(super) const CRUD_DATABASE: &str = r#"import Users from "@/server/entities/users-entity"
import Blogs from "@/server/entities/blogs-entity"
import Sessions from "@/server/entities/sessions-entity"

database appDb provider:"dowe" host:env.DOWE_HOST port:env.DOWE_PORT account:env.DOWE_USER secret:env.DOWE_PASSWORD name:env.DOWE_DATABASE entities:[Users Blogs Sessions] seeders:[]
cache appCache provider:"dowe" host:env.CACHE_HOST port:env.CACHE_PORT account:env.CACHE_USER secret:env.CACHE_PASSWORD name:env.CACHE_DATABASE
"#;

pub(super) const CRUD_USERS_REPOSITORY: &str = r#"import { appDb, appCache } from "@/server/config/database"

fn createUserRepository params:{ name:string email:string password:string }
  query user conn:appDb.insert table:"users" value:{ name:args.name email:args.email password:args.password createdAt:now } required:["name" "email" "password"]
  return value:user

fn findUserByCredentialsRepository params:{ email:string password:string }
  query user conn:appDb.read table:"users" where:{ email:args.email password:args.password } required:true
  return value:user

fn findUserByIdRepository params:{ id:string }
  query user conn:appDb.read table:"users" where:{ id:args.id } required:true
  return value:user

fn createSessionRepository params:{ userId:string }
  id session source:"ulid"
  query created conn:appDb.insert table:"sessions" value:{ id:session userId:args.userId createdAt:now } required:["id" "userId"]
  str sessionKey source:"join" values:["session" session] delimiter:":"
  kv cached conn:appCache.set key:sessionKey value:{ id:session userId:args.userId }
  return value:{ id:session userId:args.userId }

fn deleteSessionRepository params:{ id:string }
  str sessionKey source:"join" values:["session" args.id] delimiter:":"
  kv removed conn:appCache.delete key:sessionKey
  query deleted conn:appDb.delete table:"sessions" where:{ id:args.id } required:false
  return value:deleted
"#;

pub(super) const CRUD_BLOGS_REPOSITORY: &str = r#"import appDb from "@/server/config/database"
import BlogPatch from "@/server/types/blogs-types"

fn listBlogsRepository
  query blogs conn:appDb.list table:"blogs"
  return value:blogs

fn createBlogRepository params:{ title:string content:string ownerId:string }
  query created conn:appDb.insert table:"blogs" value:{ title:args.title content:args.content ownerId:args.ownerId createdAt:now updatedAt:now } required:["title" "content"]
  return value:created

fn updateBlogRepository params:{ id:string ownerId:string patch:BlogPatch }
  query updated conn:appDb.update table:"blogs" where:{ id:args.id ownerId:args.ownerId } value:{ title:args.patch.title content:args.patch.content updatedAt:now } required:true
  return value:updated
"#;

pub(super) const CRUD_USERS_SERVICE: &str = r#"import { createUserRepository, findUserByCredentialsRepository, findUserByIdRepository, createSessionRepository, deleteSessionRepository } from "@/server/repositories/users-repository"

fn registerUserService params:{ name:string email:string password:string }
  createUserRepository user args:{ name:args.name email:args.email password:args.password }
  createSessionRepository session args:{ userId:user.id }
  str authorization source:"join" values:["Bearer" session.id] delimiter:" "
  return value:{ authenticated:true guest:false authorization:authorization token:session.id user:{ id:user.id name:user.name email:user.email } }

fn loginUserService params:{ email:string password:string }
  findUserByCredentialsRepository user args:{ email:args.email password:args.password }
  createSessionRepository session args:{ userId:user.id }
  str authorization source:"join" values:["Bearer" session.id] delimiter:" "
  return value:{ authenticated:true guest:false authorization:authorization token:session.id user:{ id:user.id name:user.name email:user.email } }

fn getSessionService params:{ subject:string authorization:string token:string session:string }
  findUserByIdRepository user args:{ id:args.subject }
  return value:{ authenticated:true guest:false authorization:args.authorization token:args.token user:{ id:user.id name:user.name email:user.email } }

fn logoutUserService params:{ session:string }
  deleteSessionRepository deleted args:{ id:args.session }
  return value:{ authenticated:false guest:true authorization:"" token:"" user:{ id:"" name:"" email:"" } }
"#;

pub(super) const CRUD_BLOGS_SERVICE: &str = r#"import BlogPatch from "@/server/types/blogs-types"
import { listBlogsRepository, createBlogRepository, updateBlogRepository } from "@/server/repositories/blogs-repository"

fn listBlogsService
  listBlogsRepository blogs
  return value:blogs

fn createBlogService params:{ title:string content:string ownerId:string }
  createBlogRepository created args:{ title:args.title content:args.content ownerId:args.ownerId }
  listBlogsRepository blogs
  return value:{ blogs:blogs created:created }

fn updateBlogService params:{ id:string ownerId:string patch:BlogPatch }
  updateBlogRepository updated args:{ id:args.id ownerId:args.ownerId patch:args.patch }
  listBlogsRepository blogs
  return value:{ changed:updated.changed blogs:blogs }
"#;

pub(super) const CRUD_USERS_HANDLER: &str = r#"import { registerUserService, loginUserService, getSessionService, logoutUserService } from "@/server/services/users-service"
import { LoginInput, RegisterInput } from "@/server/types/auth-types"

handler registerUser
  const body:RegisterInput value:req.json
  registerUserService result args:{ name:body.name email:body.email password:body.password }
  return status:201 json:{ ok:true data:result }

handler loginUser
  const body:LoginInput value:req.json
  loginUserService result args:{ email:body.email password:body.password }
  return json:{ ok:true data:result }

handler getSession
  getSessionService result args:{ subject:req.context.auth.subject authorization:req.context.auth.authorization token:req.context.auth.token session:req.context.auth.session }
  return json:{ ok:true data:result }

handler logoutUser
  logoutUserService result args:{ session:req.context.auth.session }
  return json:{ ok:true data:result }
"#;

pub(super) const CRUD_BLOGS_HANDLER: &str = r#"import { createBlogService, listBlogsService, updateBlogService } from "@/server/services/blogs-service"
import { BlogInput, BlogPatch } from "@/server/types/blogs-types"

handler listBlogs
  listBlogsService result
  return json:{ ok:true data:result }

handler createBlog
  const body:BlogInput value:req.json
  createBlogService result args:{ title:body.title content:body.content ownerId:req.context.auth.subject }
  return status:201 json:{ ok:true data:result.blogs created:result.created }

handler updateBlog
  const body:BlogPatch value:req.json
  updateBlogService result args:{ id:req.params.id ownerId:req.context.auth.subject patch:body }
  return json:{ ok:true changed:result.changed data:result.blogs }
"#;

pub(super) const CRUD_AUTH_MIDDLEWARE: &str = r#"import { appDb, appCache } from "@/server/config/database"

middleware requireBearer
  bearer token value:req.header.Authorization
  session verified cache:appCache database:appDb token:token maxAge:2592000
  if verified.valid
    next context:{ auth:{ subject:verified.userId session:verified.id authorization:req.header.Authorization token:token } }
  return status:401 json:{ ok:false error:"Unauthorized" }
"#;
