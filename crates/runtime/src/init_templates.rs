use crate::init::{InitProjectOptions, ProjectTemplate, TemplateFile};

mod crud_server;

use crud_server::{
    CRUD_API_ROUTES, CRUD_AUTH_MIDDLEWARE, CRUD_AUTH_TYPES, CRUD_BLOG_TYPES, CRUD_BLOGS,
    CRUD_BLOGS_HANDLER, CRUD_BLOGS_REPOSITORY, CRUD_BLOGS_SERVICE, CRUD_DATABASE, CRUD_SESSIONS,
    CRUD_USERS, CRUD_USERS_HANDLER, CRUD_USERS_REPOSITORY, CRUD_USERS_SERVICE,
};


include!("init_template_basics.rs");
include!("init_template_translations.rs");
include!("init_template_crud_files.rs");
include!("init_template_localization.rs");
