use quote::{quote, ToTokens};
use syn::{ReturnType, TraitItem};

use super::{Entity, EntityType};

pub fn file_parser(file: syn::File) -> Vec<Entity> {
    let mut entities: Vec<Entity> = vec![];
    for item in file.items {
        match item {
            syn::Item::Enum(_item) => (),
            syn::Item::Struct(item) => entities.push(struct_parser(item)),
            syn::Item::Trait(item) => entities.push(trait_parser(item)),
            _ => (),
        }
    }
    entities
}

fn trait_parser(item: syn::ItemTrait) -> Entity {
    let name = item.ident.to_string();
    let mut methods: Vec<Entity> = vec![];

    for item in &item.items {
        if let TraitItem::Method(method) = item {
            let method_name = &method.sig.ident;
            let mut params: Vec<String> = vec![];

            // Extract input parameters
            for input in &method.sig.inputs {
                match input {
                    syn::FnArg::Receiver(_) => (),
                    syn::FnArg::Typed(pat_type) => {
                        // Get parameter name and type
                        if let syn::Pat::Ident(ident) = &*pat_type.pat {
                            params.push(format!("{} {}", ident.ident, pat_type.ty.to_token_stream().to_string()));
                        }
                    }
                }
            }

            // Handle the return type
            let ret = match &method.sig.output {
                ReturnType::Default => "".to_string(),
                ReturnType::Type(_, ty) => format!("{}", ty.to_token_stream().to_string()),
            };

            let method = format!("{}({}) {}", method_name, params.join(","), ret);

            methods.push(Entity {
                entity_type: EntityType::Field("".to_owned()),
                name: method,
                fields: vec![],
            });
        }
    }
    Entity {
        entity_type: EntityType::Trait,
        name,
        fields: methods,
    }
}

fn struct_parser(item: syn::ItemStruct) -> Entity {
    let name = item.ident.to_string();
    let fields = fields_parser(item.fields);
    Entity {
        entity_type: EntityType::Struct,
        name,
        fields,
    }
}

fn fields_parser(item: syn::Fields) -> Vec<Entity> {
    match item {
        syn::Fields::Named(syn::FieldsNamed { named: fields, .. }) => {
            fields.into_iter().map(field_parser).collect()
        }
        _ => vec![],
    }
}

fn field_parser(field: syn::Field) -> Entity {
    let name = field
        .ident
        .map(|ident| ident.to_string())
        .unwrap_or_else(|| "".to_string());

    if has_dependencies(&type_parser(field.ty.clone())) {
        let fields = make_dependencies(&type_parser(field.ty.clone()));
        return Entity::new(
            EntityType::Field(name),
            &type_parser(field.ty),
            vec![fields],
        );
    }
    Entity::new(EntityType::Field(name), &type_parser(field.ty), Vec::new())
}

fn type_parser(type_: syn::Type) -> String {
    let type_name = quote!(#type_);
    type_name
        .to_string()
        .chars()
        .filter(|c| *c != ' ')
        .collect()
}

fn has_dependencies(type_name: &str) -> bool {
    let cnt = type_name
        .chars()
        .filter(|x| (*x == ',') || (*x == '<') || (*x == '>'))
        .count();
    if cnt != 0 {
        return true;
    }
    false
}

fn make_dependencies(type_name: &str) -> Entity {
    let dependencies: Vec<&str> = type_name
        .split(|x| (x == ',') || (x == '<') || (x == '>'))
        .collect();
    let dependencies = dependencies
        .into_iter()
        .map(|x| x.to_string())
        .filter(|x| x != "")
        .map(|x| x.replace(" ", ""))
        .collect::<Vec<String>>();
    let dependencies = dependencies
        .into_iter()
        .map(|x| Entity::new(EntityType::Field("".to_string()), &x, Vec::new()))
        .collect::<Vec<Entity>>();
    Entity::new(EntityType::Struct, type_name, dependencies)
}
