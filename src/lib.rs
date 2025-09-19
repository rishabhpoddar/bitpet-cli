//! Procedural macros for automatic error tracking in bitpet-cli

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input, Item, ItemMod, ImplItem, ImplItemFn};

/// Procedural macro that automatically adds function names to error backtraces
///
/// Usage:
/// ```rust
/// #[track_errors]
/// fn my_function() -> Result<(), Box<dyn CustomErrorTrait>> {
///     some_operation()?; // Errors automatically get function name added
///     Ok(())
/// }
/// ```
///
/// This macro wraps the function body to intercept any errors and automatically
/// add the current function name to the error's backtrace using the `add_context` method.
#[proc_macro_attribute]
pub fn track_errors(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the function
    let input_fn = parse_macro_input!(item as ItemFn);

    // Extract function name and location info
    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();

    // Generate location info that will be computed at runtime
    let location_info = quote! {
        format!("{}:{} in {}", file!(), line!(), #fn_name_str)
    };

    // Extract function components
    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;
    let fn_block = &input_fn.block;

    // Check if this is an async function
    let is_async = fn_sig.asyncness.is_some();

    // Extract the function output type if present (e.g., Result<...>)
    let output_ty_opt = match &fn_sig.output {
        syn::ReturnType::Type(_, ty) => Some((**ty).clone()),
        _ => None,
    };

    // Check if the return type is a Result that could contain our CustomErrorTrait
    let has_result_return = match &fn_sig.output {
        syn::ReturnType::Type(_, ty) => {
            // This is a simplified check - in a real implementation you might want
            // more sophisticated type analysis
            quote!(#ty).to_string().contains("Result")
        }
        _ => false,
    };

    if !has_result_return {
        // If the function doesn't return a Result, just return it unchanged
        return quote!(#input_fn).into();
    }

    // Generate the wrapped function
    let output_ty = output_ty_opt.expect("Expected a return type for function");
    let expanded = if is_async {
        // For async functions
        quote! {
            #fn_vis #fn_sig {
                let __location_info = #location_info;

                async move {
                    // Execute the original function body and capture its Result with explicit type
                    let __result: #output_ty = async move #fn_block.await;

                    match __result {
                        Ok(value) => Ok(value),
                        Err(mut error) => {
                            error.add_context(__location_info);
                            Err(error)
                        }
                    }
                }.await
            }
        }
    } else {
        // For regular functions
        quote! {
            #fn_vis #fn_sig {
                let __location_info = #location_info;

                // Execute the original function body and capture its Result with explicit type
                let __result: #output_ty = (|| #fn_block)();

                match __result {
                    Ok(value) => Ok(value),
                    Err(mut error) => {
                        error.add_context(__location_info);
                        Err(error)
                    }
                }
            }
        }
    };

    TokenStream::from(expanded)
}

/// Alternative macro for more explicit control over error tracking
///
/// Usage:
/// ```rust
/// #[track_errors_with_name("custom_operation")]
/// fn complex_function_name() -> Result<(), Box<dyn CustomErrorTrait>> {
///     // Errors will show "custom_operation" instead of "complex_function_name"
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn track_errors_with_name(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the custom name from the attribute
    let custom_name = if attr.is_empty() {
        None
    } else {
        // Simple parsing - in production you'd want better error handling
        let attr_str = attr.to_string();
        Some(attr_str.trim_matches('"').to_string())
    };

    // Parse the function
    let input_fn = parse_macro_input!(item as ItemFn);

    // Use custom name or fall back to function name, and add location info
    let fn_name = &input_fn.sig.ident;
    let base_name = custom_name.unwrap_or_else(|| fn_name.to_string());

    // Generate location info that will be computed at runtime
    let location_info = quote! {
        format!("{}:{} in {}", file!(), line!(), #base_name)
    };

    // Extract function components
    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;
    let fn_block = &input_fn.block;

    // Check if this is an async function
    let is_async = fn_sig.asyncness.is_some();

    // Extract the function output type if present (e.g., Result<...>)
    let output_ty_opt = match &fn_sig.output {
        syn::ReturnType::Type(_, ty) => Some((**ty).clone()),
        _ => None,
    };

    // Check if the return type is a Result
    let has_result_return = match &fn_sig.output {
        syn::ReturnType::Type(_, ty) => quote!(#ty).to_string().contains("Result"),
        _ => false,
    };

    if !has_result_return {
        return quote!(#input_fn).into();
    }

    // Generate the wrapped function
    let output_ty = output_ty_opt.expect("Expected a return type for function");
    let expanded = if is_async {
        quote! {
            #fn_vis #fn_sig {
                let __location_info = #location_info;

                async move {
                    // Execute the original function body and capture its Result with explicit type
                    let __result: #output_ty = async move #fn_block.await;

                    match __result {
                        Ok(value) => Ok(value),
                        Err(mut error) => {
                            error.add_context(__location_info);
                            Err(error)
                        }
                    }
                }.await
            }
        }
    } else {
        quote! {
            #fn_vis #fn_sig {
                let __location_info = #location_info;

                // Execute the original function body and capture its Result with explicit type
                let __result: #output_ty = (|| #fn_block)();

                match __result {
                    Ok(value) => Ok(value),
                    Err(mut error) => {
                        error.add_context(__location_info);
                        Err(error)
                    }
                }
            }
        }
    };

    TokenStream::from(expanded)
}
