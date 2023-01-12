use proc_macro::TokenStream;
use quote::quote;
use syn::{self, Data, Fields, DeriveInput, PathArguments, Type };

#[proc_macro_derive(DatabaseFactory)]
pub fn databasefactory_derive(input: TokenStream) -> TokenStream
{
    // Build an expression tree from the tokens   
    let tokens: DeriveInput = syn::parse(input).unwrap();
    
    let mut expression = quote! {};
    
    if let Data::Struct(ds) = &tokens.data
    {
        let struct_name = &tokens.ident;
                
        if let Fields::Named(fields) = &ds.fields
        {
            // Generate the expression for all fields
            let field_expressions = fields.named.iter().map(|field|
                {                    
                    // Get field name and type to use in the quote tamplte
                    let field_name = &field.ident;
                    let field_type = &field.ty;

                    // Generate expression for one field
                    quote! { #field_name: #field_type::new(std::stringify!(#field_name), transaction_manager_ref.clone()) }
                }
            );            

            // Generate the expressions 
            expression = quote! {
                impl DatabaseFactory for #struct_name
                {
                    fn create_database(transaction_manager_ref: std::sync::Arc<std::sync::Mutex<microdb::transaction::TransactionManager>>) -> Self
                    {        
                        return Self
                        {                            
                            #(#field_expressions),*
                        }
                    }
                }
            };            
        }        
    }
    else
    {
        panic!("Only structs are supported by DatabaseFactory implementation");
    } 

    return expression.into();
}

#[proc_macro_derive(Database)]
pub fn database_derive(input: TokenStream) -> TokenStream
{
    // Build an expression tree from the tokens   
    let tokens: DeriveInput = syn::parse(input).unwrap();
    
    let mut expression = quote! {};
    
    if let Data::Struct(ds) = &tokens.data
    {
        let struct_name = &tokens.ident;
                
        if let Fields::Named(fields) = &ds.fields
        {
            // Generate the expression for all fields
            let field_expressions = fields.named.iter().map(|field|
                {                    
                    // Get field name and type to use in the quote tamplte
                    let field_name = &field.ident;

                    // Generate expression for one field                    
                    quote! { if table_id == self.#field_name.get_id() { return &mut self.#field_name }; }
                }
            );            

            // Generate the expressions 
            expression = quote! {
                impl Database for #struct_name
                {
                    fn get_table_mut(&mut self, table_id: u64) -> &mut dyn microdb::table::TableBase
                    {                               
                        #(#field_expressions)*
                        panic!("Unknown table");
                    }
                }
            };            
        }        
    }
    else
    {
        panic!("Only structs are supported by DatabaseFactory implementation");
    } 

    return expression.into();
}

#[proc_macro_derive(CommandDirectory)]
pub fn commanddefinitions_derive(input: TokenStream) -> TokenStream
{
    // Build an expression tree from the tokens   
    let tokens: DeriveInput = syn::parse(input).unwrap();
    
    let mut expression = quote! {};
    
    if let Data::Struct(ds) = &tokens.data
    {
        let struct_name = &tokens.ident;
                
        if let Fields::Named(fields) = &ds.fields
        {
            let field = fields.named.first().unwrap();
            let mut database_type = None;

            match &field.ty
                    {
                        Type::Path(path) => {
                             let arguments = &path.path.segments[0].arguments;
                             if let PathArguments::AngleBracketed(args) = arguments
                             {                       
                                database_type = Some(args.args.first().unwrap());                                
                             }
                            },
                        _ => {}
                    }

            // Generate the expression for all fields
            let field_expressions = fields.named.iter().map(|field|
                { 
                    // Get field name and type to use in the quote tamplte
                    let field_name = &field.ident;
                    let field_type = &field.ty;                   

                    // Generate expression for one field
                    quote! { std::stringify!(#field_name) => Box::new(#field_type::new(self.#field_name.get_name(), self.#field_name.get_cmd()))}
                }
            );            

            // Generate the expressions 
            expression = quote! {
                impl CommandDirectory<#database_type> for #struct_name
                {
                    fn get(&self, name: &str) -> Box<dyn microdb::command::CommandDefinitionBase<#database_type>>
                    {
                        match name
                        {                               
                            #(#field_expressions),*,
                            _s => panic!("Unknown command {}", _s)                    
                        }
                    }
                }
            };            
        }        
    }
    else
    {
        panic!("Only structs are supported by DatabaseFactory implementation");
    } 

    return expression.into();    
}

#[proc_macro_derive(CommandDirectoryFactory)]
pub fn commanddirectoryfactory_derive(input: TokenStream) -> TokenStream
{
    // Build an expression tree from the tokens   
    let tokens: DeriveInput = syn::parse(input).unwrap();
    
    let mut expression = quote! {};
    
    if let Data::Struct(ds) = &tokens.data
    {
        let struct_name = &tokens.ident;
                
        if let Fields::Named(fields) = &ds.fields
        {
            // Generate the expression for all fields
            let field_expressions = fields.named.iter().map(|field|
                {                    
                    // Get field name and type to use in the quote tamplte
                    let field_name = &field.ident;
                    //let field_type = &field.ty;

                    // Generate expression for one field
                    quote! { #field_name: microdb::command::CommandDefinition::new(std::stringify!(#field_name), #struct_name::#field_name) }
                }
            );            

            // Generate the expressions 
            expression = quote! {
                impl CommandDirectoryFactory for #struct_name
                {
                    fn new() -> Self
                    {        
                        return Self
                        {                            
                            #(#field_expressions),*
                        }
                    }
                }
            };            
        }        
    }
    else
    {
        panic!("Only structs are supported by DatabaseFactory implementation");
    } 

    return expression.into();
}