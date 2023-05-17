module pool_manager::utilities {
    use aptos_framework::object::Object;
    use aptos_framework::fungible_asset::Metadata;
    use std::string::String;
    use aptos_framework::fungible_asset;
    use std::string;

    friend pool_manager::liquidity_pool;

    public(friend) fun swap_lp_token_name_and_symbol(x: Object<Metadata>, y: Object<Metadata>): (String, String) {
        let name = fungible_asset::name(x);
        string::append_utf8(&mut name, b"<>");
        string::append(&mut name, fungible_asset::name(y));

        let symbol = fungible_asset::symbol(x);
        string::append_utf8(&mut symbol, b"/");
        string::append(&mut symbol, fungible_asset::symbol(y));
        (name, symbol)
    }
}
