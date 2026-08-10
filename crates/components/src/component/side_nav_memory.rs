pub fn side_nav_memory_key(props: &SideNavProps, items: &[SideNavItem]) -> String {
    if let Some(id) = props.style.element.id.as_deref() {
        return format!("id:{id}");
    }
    let mut hash = 0xcbf29ce484222325_u64;
    hash_side_nav_items(&mut hash, items);
    format!("structure:{hash:016x}")
}

fn hash_side_nav_items(hash: &mut u64, items: &[SideNavItem]) {
    hash_bytes(hash, items.len().to_string().as_bytes());
    for item in items {
        match item {
            SideNavItem::Header(props) => {
                hash_bytes(hash, b"header");
                hash_side_nav_item_props(hash, props);
            }
            SideNavItem::Item(props) => {
                hash_bytes(hash, b"item");
                hash_side_nav_item_props(hash, props);
            }
            SideNavItem::Divider => hash_bytes(hash, b"divider"),
            SideNavItem::Submenu {
                props,
                bordered,
                items,
                ..
            } => {
                hash_bytes(hash, b"submenu");
                hash_side_nav_item_props(hash, props);
                hash_bytes(hash, if *bordered { b"bordered" } else { b"plain" });
                for item in items {
                    hash_side_nav_item_props(hash, item);
                }
            }
        }
    }
}

fn hash_side_nav_item_props(hash: &mut u64, props: &SideNavItemProps) {
    for value in [
        Some(props.label.as_str()),
        props.description.as_deref(),
        props.status.as_deref(),
        props.on_click.as_deref(),
    ] {
        match value {
            Some(value) => {
                hash_bytes(hash, b"some");
                hash_bytes(hash, value.as_bytes());
            }
            None => hash_bytes(hash, b"none"),
        }
    }
    if let Some(navigation) = props.navigation.as_ref() {
        hash_bytes(hash, format!("{navigation:?}").as_bytes());
    } else {
        hash_bytes(hash, b"no-navigation");
    }
}

fn hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x100000001b3);
    }
    *hash ^= 0xff;
    *hash = hash.wrapping_mul(0x100000001b3);
}
