use proc_macro2::TokenStream;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    Attribute, Generics, Ident, Token, Type, TypeParamBound, Visibility, WhereClause,
};

// https://github.com/intellij-rust/intellij-rust/issues/6236
#[allow(unused_imports)]
use syn::token::Token;

#[proc_macro]
pub fn trait_union(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let TraitUnionRequests(requests) = parse_macro_input!(tokens as TraitUnionRequests);
    let mut tokens = TokenStream::new();
    for request in requests {
        tokens.extend(handle_request(request));
    }
    tokens.into()
}

struct TraitUnionRequest {
    attr: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    generics: Generics,
    trait_: Punctuated<TypeParamBound, Token![+]>,
    variants: Punctuated<Type, Token![|]>,
}

impl Parse for TraitUnionRequest {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attr = input.call(Attribute::parse_outer)?;
        let vis = input.parse::<Visibility>()?;
        let _t_union = input.parse::<Token![union]>()?;
        let ident = input.parse::<Ident>()?;
        let mut generics = input.parse::<Generics>()?;
        let _t_colon = input.parse::<Token![:]>()?;
        let mut trait_ = Punctuated::new();
        loop {
            trait_.push_value(input.parse()?);
            if input.peek(Token![where]) || input.peek(Token![=]) {
                break;
            }
            trait_.push_punct(input.parse()?);
            if input.peek(Token![where]) || input.peek(Token![=]) {
                break;
            }
        }
        if input.peek(Token![where]) {
            generics.where_clause = Some(input.parse::<WhereClause>()?);
        }
        let _t_equals = input.parse::<Token![=]>()?;
        let mut variants = Punctuated::new();
        loop {
            variants.push_value(input.parse::<Type>()?);
            if !input.peek(Token![|]) {
                break;
            }
            variants.push_punct(input.parse::<Token![|]>()?);
            if input.peek(Token![;]) {
                break;
            }
        }
        let _t_semicolon = input.parse::<Token![;]>()?;
        Ok(TraitUnionRequest {
            attr,
            vis,
            ident,
            generics,
            trait_,
            variants,
        })
    }
}

struct TraitUnionRequests(Vec<TraitUnionRequest>);

impl Parse for TraitUnionRequests {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut res = vec![];
        while !input.is_empty() {
            res.push(TraitUnionRequest::parse(input)?);
        }
        Ok(TraitUnionRequests(res))
    }
}

fn handle_request(request: TraitUnionRequest) -> TokenStream {
    let attr = request.attr;
    let vis = request.vis;
    let name = request.ident;
    let name_as_str = format!("{}", name);
    let prefix = format!("__trait_union_{}_", name);
    let data_name = Ident::new(&format!("{}data", prefix), name.span());
    let vtable_name = Ident::new(&format!("{}vtable", prefix), name.span());
    let variant_name = Ident::new(&format!("{}Variant", name), name.span());
    let union_name = Ident::new(&format!("{}Union", prefix), name.span());
    let trait_object_name = Ident::new(&format!("{}TraitObject", prefix), name.span());
    let vtable_container_name = Ident::new(&format!("{}VtableContainer", prefix), name.span());
    let to_trait_object_name =
        Ident::new(&format!("{}to_trait_object", prefix), name.span());
    let mut trait_ = request.trait_;
    let has_lifetime = trait_
        .iter()
        .any(|b| matches!(b, TypeParamBound::Lifetime(_)));
    if !has_lifetime {
        if !trait_.empty_or_trailing() {
            trait_.push_punct(syn::token::Add(trait_.span()));
        }
        trait_.push_value(TypeParamBound::Lifetime(syn::Lifetime::new(
            "'static",
            trait_.span(),
        )));
    }
    let (impl_generics, ty_generics, where_clause) = request.generics.split_for_impl();
    let mut union_fields = vec![];
    for (pos, variant) in request.variants.iter().enumerate() {
        let ident = Ident::new(&format!("variant{}", pos), variant.span());
        union_fields.push(
            quote::quote_spanned!( variant.span() => #ident: core::mem::ManuallyDrop<#variant>),
        );
    }
    let mut variant_impls = vec![];
    for variant in &request.variants {
        variant_impls.push(quote::quote_spanned! { variant.span() =>
            unsafe impl#impl_generics #variant_name#ty_generics for #variant #where_clause { }
        })
    }
    let tokens = quote::quote! {
        #(#attr)*
        #[allow(non_snake_case)]
        #vis struct #name#impl_generics #where_clause {
            #data_name: #union_name#ty_generics,
            #vtable_name: #vtable_container_name,
        }

        /// Marker trait for types that can be stored in a [
        #[doc = #name_as_str]
        ///]
        ///
        /// # Safety
        ///
        /// This trait must not be implemented manually.
        #vis unsafe trait #variant_name#impl_generics: #trait_ {}

        #[repr(C)]
        #[allow(non_snake_case)]
        struct #trait_object_name {
            data: *mut (),
            vtable: *mut (),
        }

        #[repr(C)]
        #[allow(non_snake_case)]
        union #union_name#impl_generics #where_clause {
            #(#union_fields),*
        }

        #[allow(non_camel_case_types)]
        struct #vtable_container_name(core::ptr::NonNull<()>);
        unsafe impl core::marker::Send for #vtable_container_name { }
        unsafe impl core::marker::Sync for #vtable_container_name { }

        impl#impl_generics #name#ty_generics #where_clause {
            /// Creates a new instance
            #[inline(always)]
            #vis fn new(value: impl #variant_name#ty_generics) -> Self {
                let mut slf = core::mem::MaybeUninit::<Self>::uninit();
                let vtable = {
                    let trait_object: &(dyn #trait_) = &value;
                    let trait_object: #trait_object_name = unsafe { core::mem::transmute(trait_object) };
                    trait_object.vtable
                };
                unsafe {
                    core::ptr::write(&mut (*slf.as_mut_ptr()).#data_name as *mut _ as *mut _, value);
                    (*slf.as_mut_ptr()).#vtable_name = #vtable_container_name(core::ptr::NonNull::new_unchecked(vtable));
                    slf.assume_init()
                }
            }
        }

        #[inline(always)]
        #[allow(non_snake_case)]
        fn #to_trait_object_name#impl_generics(x: &#name#ty_generics) -> #trait_object_name #where_clause {
            #trait_object_name {
                data: &x.#data_name as *const _ as *mut _,
                vtable: x.#vtable_name.0.as_ptr(),
            }
        }

        impl#impl_generics core::ops::Drop for #name#ty_generics #where_clause {
            #[inline(always)]
            fn drop(&mut self) {
                unsafe {
                    let t: &mut (dyn #trait_) = core::mem::transmute(#to_trait_object_name(self));
                    core::ptr::drop_in_place(t);
                }
            }
        }

        impl#impl_generics core::ops::Deref for #name#ty_generics #where_clause {
            type Target = dyn #trait_;

            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                unsafe { core::mem::transmute(#to_trait_object_name(self)) }
            }
        }

        impl#impl_generics core::ops::DerefMut for #name#ty_generics #where_clause {
            #[inline(always)]
            fn deref_mut(&mut self) -> &mut Self::Target {
                unsafe { core::mem::transmute(#to_trait_object_name(self)) }
            }
        }

        #(#variant_impls)*
    };
    tokens
}
