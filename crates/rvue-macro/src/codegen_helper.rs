fn generate_children_code(children: &[RvueNode], parent_id: &Ident) -> TokenStream {
    let child_vars = children.iter().map(|child| {
        let child_code = generate_node_code(child);
        quote! {
            let child = #child_code;
            #parent_id.add_child(child);
        }
    });

    quote! {
        #(#child_vars)*
    }
}
