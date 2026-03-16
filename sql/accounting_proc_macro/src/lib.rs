use std::path::PathBuf;

// use proc_macro::{Ident, TokenStream};
use quote::quote;
use syn::{Fields, parse::Parse, parse_macro_input};

const SQL_ROOT: &str = "src/scripts";

fn sqlite_scripts_path() -> PathBuf {
    std::path::Path::new(SQL_ROOT).join("sqlite")
}

#[proc_macro_attribute]
pub fn sqlite_impl(attributes: proc_macro::TokenStream,item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // let input = proc_macro2::TokenStream::from(input);
    let input = parse_macro_input!(item as syn::ItemStruct);
    let ty = &input.ident;
    let ty_lowercase = ty.to_string().to_ascii_lowercase();

    let ty_sql_path = sqlite_scripts_path().join(ty.to_string().to_lowercase());
    let ty_get_one_sql_path = ty_sql_path
        .join("get_one.sql")
        .to_string_lossy()
        .to_string();
    let ty_get_many = ty_sql_path
        .join("get_many.sql")
        .to_string_lossy()
        .to_string();
    let ty_insert = ty_sql_path.join("insert.sql").to_string_lossy().to_string();
    let ty_insert_many = ty_sql_path
        .join("insert_many.sql")
        .to_string_lossy()
        .to_string();
    let ty_insert_with_returning = ty_sql_path
        .join("insert_with_returning.sql")
        .to_string_lossy()
        .to_string();
    let ty_update = ty_sql_path.join("update.sql").to_string_lossy().to_string();



    // the update query_as
    let mut query_as = format!("sqlx_query_as!({ty},{ty_insert}");
    let fields = match &input.fields {
        Fields::Named(f) =>&f.named,
        _=>{return  quote!{compile_error!("struct #ty only named fields are supported")}.into();}
    };
    for field in fields {
        query_as.push_str(&format!(",t.{}",field.ident.as_ref().unwrap()));;
    }
    query_as.push_str(")");

    let e = quote! {
        #input

        impl #ty {
            pub async fn async_sqlite_get_one<'a,E>(executor:E,id:crate::Id<#ty>) -> Result<#ty,Box<dyn std::error::Error>>
                where E: sqlx::SqliteExecutor<'a> {
                let t = sqlx::query_file_as!(#ty,#ty_get_one_sql_path,id).fetch_one(executor).await?;
                Ok(t)
            }

            pub async fn async_sqlite_insert<'a,E>(executor:E,#ty_lowercase: &#ty) -> Result<#ty,Box<dyn std::error::Error>>
                where E: sqlx::SqliteExecutor<'a>,
                #ty: GetId + Sized {
                let id = #ty.get_id();
                // let t = sqlx::query_file_as!(#ty,#ty_insert,id).fetch_many(executor).await?;
                // Ok(t)
                todo!()
            }
            // pub async fn async_sqlite_insert_with_returning<'a,E>(executor:E,id:crate::Id<#ty>) -> Result<#ty,Box<dyn std::error::Error>>
            //     where E: sqlx::SqliteExecutor<'a> {
            //     let t = sqlx::query_file_as!(#ty,#ty_insert_with_returning,id).fetch_many(executor).await?;
            //     Ok(t)
            // }
            // pub async fn async_sqlite_update<'a,E>(executor:E,id:crate::Id<#ty>) -> Result<#ty,Box<dyn std::error::Error>>
            //     where E: sqlx::SqliteExecutor<'a> {
            //     let t = sqlx::query_file_as!(#ty,#ty_update,id).fetch_many(executor).await?;
            //     Ok(t)
            // }

        }

    };
    e.into()
}
