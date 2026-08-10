    #[test]
    fn parses_display_and_overlay_view_components() {
        let tree = parse_page(
            r#"page overlayPage
  signal modalOpen value:false
  signal toast value:{ type:"success" title:"Saved" message:"Profile updated" visible:true }
  fn close
    reset modalOpen
  Box
    Avatar name:"Ada" alt:"Ada Lovelace" scheme:"success" variant:"soft" size:"lg" status:"online" bordered:true
    Badge text:"3" scheme:"danger" position:"bottom-right"
      Avatar name:"Ada" alt:"Ada"
    Chip variant:"outlined" scheme:"info" size:"sm" onClose:close
      Filter
    Skeleton variant:"rounded" animation:"pulse"
    Modal open:modalOpen scheme:"surface" hideCloseButton:true
      header
        Title
          "Settings"
      Text
        "Body"
      footer
        Button onClick:close
          "Close"
    AlertDialog open:modalOpen title:"Delete?" description:"Cannot undo." confirmText:"Delete" cancelText:"Cancel" onConfirm:close onCancel:close scheme:"danger"
    Tooltip label:"More actions" position:"end" scheme:"muted"
      Text
        "Hover"
    Toast source:toast position:"top-right" showIcon:true
    Dropdown scheme:"surface"
      trigger
        Button
          "Menu"
      item label:"Profile" onClick:close
      divider
      item label:"Docs" href:"/docs" description:"Open docs"
    Command open:modalOpen placeholder:"Search" shortcut:"p" scheme:"muted"
      item label:"Back" history:"back"
      group label:"Admin"
        item label:"Users" onClick:close"#,
        )
        .expect("tree");

        let ViewNode::Scope { children, .. } = tree else {
            panic!("scope");
        };
        let ViewNode::Box {
            children: box_children,
            ..
        } = &children[0]
        else {
            panic!("box");
        };
        assert_eq!(box_children.len(), 10);

        let ViewNode::Avatar { props, .. } = &box_children[0] else {
            panic!("avatar");
        };
        assert_eq!(props.style.color, Some(ColorFamily::Success));
        assert_eq!(props.style.variant, Some(ComponentVariant::Soft));
        assert_eq!(props.size, ButtonSize::Lg);
        assert_eq!(props.status, Some(AvatarStatus::Online));
        assert!(props.bordered);

        let ViewNode::Badge {
            props,
            children: badge_children,
        } = &box_children[1]
        else {
            panic!("badge");
        };
        assert_eq!(props.position, OverlayCornerPosition::BottomRight);
        assert_eq!(badge_children.len(), 1);

        let ViewNode::Chip { props, value, .. } = &box_children[2] else {
            panic!("chip");
        };
        assert_eq!(value, "Filter");
        assert_eq!(props.on_close.as_deref(), Some("close"));

        let ViewNode::Skeleton { props } = &box_children[3] else {
            panic!("skeleton");
        };
        assert_eq!(props.variant, SkeletonVariant::Rounded);
        assert_eq!(props.animation, SkeletonAnimation::Pulse);

        let ViewNode::Modal {
            props,
            header,
            body,
            footer,
        } = &box_children[4]
        else {
            panic!("modal");
        };
        assert_eq!(props.open, "modalOpen");
        assert!(props.hide_close_button);
        assert_eq!(header.len(), 1);
        assert_eq!(body.len(), 1);
        assert_eq!(footer.len(), 1);

        let ViewNode::AlertDialog { props } = &box_children[5] else {
            panic!("dialog");
        };
        assert_eq!(props.on_confirm.as_deref(), Some("close"));
        assert_eq!(props.on_cancel.as_deref(), Some("close"));

        let ViewNode::Tooltip { props, children } = &box_children[6] else {
            panic!("tooltip");
        };
        assert_eq!(props.position, OverlayPosition::End);
        assert_eq!(children.len(), 1);

        let ViewNode::Toast { props } = &box_children[7] else {
            panic!("toast");
        };
        assert_eq!(props.source.as_deref(), Some("toast"));
        assert_eq!(props.kind, ToastKind::Info);
        assert_eq!(props.position, OverlayCornerPosition::TopRight);
        assert!(props.show_icon);

        let ViewNode::Dropdown { entries, .. } = &box_children[8] else {
            panic!("dropdown");
        };
        assert!(matches!(entries[0], OverlayEntry::Item(_)));
        assert!(matches!(entries[1], OverlayEntry::Divider));

        let ViewNode::Command { props, entries } = &box_children[9] else {
            panic!("command");
        };
        assert_eq!(props.open.as_deref(), Some("modalOpen"));
        assert_eq!(props.shortcut, "p");
        assert!(matches!(
            &entries[0],
            CommandEntry::Item(props)
                if matches!(props.navigation, Some(NavigationAction::Back))
        ));
        assert!(matches!(entries[1], CommandEntry::Group { .. }));
    }

    #[test]
    fn parses_display_chat_and_motion_components() {
        let tree = parse_page(
            r#"type Person
  src:string
  name:string
  alt:string

type ChatMessage
  id:string
  role:string
  userId:string
  text:string
  status:string

page displayPage
  signal people type:Person[] value:[{ src:"/ada.png" name:"Ada" alt:"Ada Lovelace" }]
  signal messages type:ChatMessage[] value:[{ id:"1" role:"assistant" userId:"assistant" text:"Hello" status:"sent" }]
  signal loading value:false
  fn sendMessage
    reset loading
  Box
    AvatarGroup items:people size:"sm" max:3 autoFit:true inline:false bordered:true scheme:"primary" variant:"soft"
      item src:"/ada.png" name:"Ada" alt:"Ada Lovelace" onClick:sendMessage
      item name:"Grace" alt:"Grace Hopper" href:"/docs"
    ChatBox messages:messages mode:"conversation" currentUserId:"ada" userName:"Ada" userAvatar:"/ada.png" userStatus:"online" assistantName:"Dowe" assistantAvatar:"/dowe.png" showHeader:true placeholder:"Ask Dowe" showAttachments:true showVoiceNote:true showCamera:true loading:loading sending:loading streaming:loading hasMore:loading onSend:sendMessage onLoadMore:sendMessage onStop:sendMessage onVoiceNote:sendMessage onFileAttach:sendMessage onCameraCapture:sendMessage
    Empty type:"result" title:"Nothing found" description:"Try again" actionLabel:"Retry" onClick:sendMessage scheme:"info" variant:"soft"
    Marquee speed:"fast" pauseOnHover:true reverse:true orientation:"horizontal" fade:true fadeColor:"background" gap:4
      Text
        "Moving"
    TypeWriter typeSpeed:10 deleteSpeed:5 afterTyped:20 afterDeleted:10 repeat:false
      item text:"Hello"
      item text:"World""#,
        )
        .expect("tree");

        let ViewNode::Scope { children, .. } = tree else {
            panic!("scope");
        };
        let ViewNode::Box {
            children: box_children,
            ..
        } = &children[0]
        else {
            panic!("box");
        };
        assert_eq!(box_children.len(), 5);

        let ViewNode::AvatarGroup { props, items } = &box_children[0] else {
            panic!("avatar group");
        };
        assert_eq!(props.items.as_deref(), Some("people"));
        assert_eq!(props.size, ButtonSize::Sm);
        assert_eq!(props.max, Some(3));
        assert!(props.auto_fit);
        assert!(!props.inline);
        assert!(props.bordered);
        assert_eq!(props.style.variant, Some(ComponentVariant::Soft));
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].on_click.as_deref(), Some("sendMessage"));
        assert!(items[1].navigation.is_some());

        let ViewNode::ChatBox { props } = &box_children[1] else {
            panic!("chat box");
        };
        assert_eq!(props.messages, "messages");
        assert_eq!(props.mode, ChatBoxMode::Conversation);
        assert_eq!(props.current_user_id, "ada");
        assert_eq!(props.loading.as_deref(), Some("loading"));
        assert_eq!(props.on_send.as_deref(), Some("sendMessage"));
        assert!(props.show_attachments);
        assert!(props.show_voice_note);
        assert!(props.show_camera);

        let ViewNode::Empty { props } = &box_children[2] else {
            panic!("empty");
        };
        assert_eq!(props.kind, EmptyKind::Result);
        assert_eq!(props.title.as_deref(), Some("Nothing found"));
        assert_eq!(props.action_label, "Retry");

        let ViewNode::Marquee { props, children } = &box_children[3] else {
            panic!("marquee");
        };
        assert_eq!(props.speed, MarqueeSpeed::Fast);
        assert_eq!(props.orientation, MarqueeOrientation::Horizontal);
        assert!(props.pause_on_hover);
        assert!(props.reverse);
        assert!(props.fade);
        assert_eq!(props.fade_color, ColorToken::Background);
        assert_eq!(children.len(), 1);

        let ViewNode::TypeWriter { props, items } = &box_children[4] else {
            panic!("type writer");
        };
        assert_eq!(props.type_speed, 10);
        assert_eq!(props.delete_speed, 5);
        assert!(!props.repeat);
        assert_eq!(items.len(), 2);
        assert_eq!(items[1].text, "World");
    }

    #[test]
    fn parses_rich_control_map_components() {
        let tree = parse_page(
            r#"page componentsPage
  signal mode value:"map"
  fn choose
    reset mode
  fn done
    reset mode
  Box
    RichText title:true size:"lg" weight:"bold"
      mark text:"Launch" style:"grad" scheme:"primary"
      mark text:"ready" style:"pill" scheme:"success"
    Record name:"voice" maxDuration:90 onStart:choose onConfirm:done variant:"soft" scheme:"primary"
    ToggleGroup value:mode selected:"map" size:"sm" wide:true ariaLabel:"Display mode" onChange:choose variant:"soft" scheme:"secondary"
      item id:"list" label:"List" icon:"search"
      item id:"map" label:"Map" icon:"settings"
    Collapsible label:"Details" defaultOpen:true scheme:"surface"
      Text
        "Body"
    Countdown target:"2030-01-01T00:00:00Z" size:"xl" showDays:true showHours:true showMinutes:true showSeconds:false onComplete:done scheme:"primary" variant:"outlined"
    Map centerLat:4.7109 centerLng:-74.0721 zoom:12 height:"360px" width:"100%" showScale:true showLocationControl:true routeStartLat:4.7109 routeStartLng:-74.0721 routeEndLat:4.65 routeEndLng:-74.09 onRoute:done scheme:"primary" variant:"soft"
      marker id:"office" lat:4.7109 lng:-74.0721 label:"Office" popup:"Main" icon:"start" onClick:choose
      waypoint lat:4.68 lng:-74.08"#,
        )
        .expect("tree");

        let ViewNode::Scope { children, .. } = tree else {
            panic!("scope");
        };
        let ViewNode::Box {
            children: box_children,
            ..
        } = &children[0]
        else {
            panic!("box");
        };
        assert_eq!(box_children.len(), 6);

        let ViewNode::RichText { props, marks } = &box_children[0] else {
            panic!("rich text");
        };
        assert_eq!(
            props
                .size
                .as_ref()
                .map(|value| value.entries[0].value.as_str()),
            Some("lg")
        );
        assert_eq!(marks.len(), 2);
        assert!(props.title);
        assert_eq!(marks[0].style, RichTextMarkStyle::Grad);
        assert_eq!(marks[1].color, ColorFamily::Success);

        let ViewNode::Record { props } = &box_children[1] else {
            panic!("record");
        };
        assert_eq!(props.name, "voice");
        assert_eq!(props.max_duration, Some(90));
        assert_eq!(props.on_start.as_deref(), Some("choose"));
        assert_eq!(props.on_confirm.as_deref(), Some("done"));
        assert_eq!(props.style.variant, Some(ComponentVariant::Soft));

        let ViewNode::ToggleGroup { props, items } = &box_children[2] else {
            panic!("toggle group");
        };
        assert_eq!(props.value.as_deref(), Some("mode"));
        assert_eq!(props.selected, "map");
        assert_eq!(props.size, ButtonSize::Sm);
        assert!(props.wide);
        assert_eq!(props.on_change.as_deref(), Some("choose"));
        assert_eq!(items.len(), 2);
        assert_eq!(items[1].icon, Some(ViewIcon::Settings));

        let ViewNode::Collapsible { props, children } = &box_children[3] else {
            panic!("collapsible");
        };
        assert_eq!(props.label, "Details");
        assert!(props.default_open);
        assert_eq!(props.style.color, Some(ColorFamily::Surface));
        assert_eq!(children.len(), 1);

        let ViewNode::Countdown { props } = &box_children[4] else {
            panic!("countdown");
        };
        assert_eq!(props.target, "2030-01-01T00:00:00Z");
        assert_eq!(props.size, CountdownSize::Xl);
        assert!(!props.show_seconds);
        assert_eq!(props.on_complete.as_deref(), Some("done"));

        let ViewNode::Map {
            props,
            markers,
            waypoints,
        } = &box_children[5]
        else {
            panic!("map");
        };
        assert_eq!(props.center_lat, "4.7109");
        assert_eq!(props.center_lng, "-74.0721");
        assert_eq!(props.zoom, 12);
        assert!(props.show_scale);
        assert!(props.show_location_control);
        assert_eq!(props.on_route.as_deref(), Some("done"));
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].lng, "-74.0721");
        assert_eq!(markers[0].icon, MapMarkerIcon::Start);
        assert_eq!(markers[0].on_click.as_deref(), Some("choose"));
        assert_eq!(waypoints.len(), 1);
        assert_eq!(waypoints[0].lng, "-74.08");
    }
