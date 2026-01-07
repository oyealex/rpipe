use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, DataEnum, Fields};

fn extract_doc(attrs: &[syn::Attribute]) -> String {
    let mut lines = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let syn::Meta::NameValue(nv) = &attr.meta {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &nv.value {
                    lines.push(s.value());
                }
            }
        }
    }
    lines.join("\n")
}

#[proc_macro_derive(CmdHelp)]
pub fn cmd_help_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;

    let Data::Enum(DataEnum { variants, .. }) = &input.data else {
        panic!("CmdHelp can only be derived on enums");
    };

    // ==============================
    // 1. 生成 help(&self) 方法
    // ==============================
    let help_entries = variants.iter().map(|v| {
        let variant_name = &v.ident;
        let doc = extract_doc(&v.attrs);
        let pattern = match &v.fields {
            Fields::Unit => quote! { #variant_name },
            Fields::Unnamed(fields) => {
                let wildcards = std::iter::repeat(quote! { _ }).take(fields.unnamed.len());
                quote! { #variant_name(#(#wildcards),*) }
            }
            Fields::Named(fields) => {
                let field_names = fields.named.iter().map(|f| &f.ident);
                quote! { #variant_name { #(#field_names: _),* } }
            }
        };
        quote! {
            Self::#pattern => #doc,
        }
    });

    // ===========================================
    // 2. 生成 all_help()：返回所有 (name, doc) 对
    // ===========================================
    let all_help_entries = variants.iter().map(|v| {
        let name = v.ident.to_string();
        let doc = extract_doc(&v.attrs);
        quote! {
            (#name, #doc)
        }
    });

    let expanded = quote! {
        impl #enum_name {
            /// 获取帮助信息。
            pub fn help(&self) -> &'static str {
                match self {
                    #(#help_entries)*
                }
            }

            /// 获取全部帮助信息：[(name, help), ...]
            pub fn all_help() -> &'static [(&'static str, &'static str)] {
                &[
                    #(#all_help_entries),*
                ]
            }
        }
    };

    TokenStream::from(expanded)
}