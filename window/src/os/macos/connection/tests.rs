use super::*;
use objc::runtime::BOOL;

#[test]
fn the_window_tab_indexer_defaults_to_off() {
    register_app_defaults();

    unsafe {
        let defaults: id = msg_send![class!(NSUserDefaults), standardUserDefaults];
        let key = nsstring("NSEnableWindowTabIndexing");
        let value: id = msg_send![defaults, objectForKey: *key];
        assert!(
            !value.is_null(),
            "NSEnableWindowTabIndexing is not registered"
        );
        let enabled: BOOL = msg_send![defaults, boolForKey: *key];
        assert_eq!(enabled, NO);
    }
}
