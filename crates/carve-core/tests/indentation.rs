use carve_core::blocks::parse_blocks;

#[test]
fn each_trigger_sets_its_own_indent_width() {
    let src = "on load:\n  loop all players:\n    send \"a\"\n\non place:\n    if event-block is emerald:\n        cancel event\n    if player's gamemode is not creative:\n        cancel event\ncommand /x:\n\ttrigger:\n\t\tsend \"b\"\n";
    let (lines, errors) = parse_blocks("t.carve", src);
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[1].children.len(), 2);
    assert_eq!(lines[1].children[0].children.len(), 1);
    assert_eq!(lines[2].children[0].children.len(), 1);
}

#[test]
fn uneven_widths_inside_a_block_follow_the_open_levels() {
    let src = "on join:\n   if true:\n         send \"deep\"\n   send \"back\"\n";
    let (lines, errors) = parse_blocks("t.carve", src);
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(lines[0].children.len(), 2);
    assert_eq!(lines[0].children[0].children.len(), 1);
}

#[test]
fn a_dedent_that_matches_no_level_is_an_error() {
    let src = "on join:\n    if true:\n        send \"a\"\n      send \"b\"\n";
    let (_, errors) = parse_blocks("t.carve", src);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].to_string().contains("doesn't line up"), "{errors:?}");
}

#[test]
fn indenting_under_a_plain_line_is_still_an_error() {
    let src = "on join:\n    send \"a\"\n        send \"b\"\n";
    let (_, errors) = parse_blocks("t.carve", src);
    assert_eq!(errors.len(), 1);
    assert!(
        errors[0].to_string().contains("unexpected indentation"),
        "{errors:?}"
    );
}

#[test]
fn mixing_tabs_and_spaces_in_one_block_is_an_error() {
    let src = "on join:\n    send \"a\"\n\tsend \"b\"\n";
    let (_, errors) = parse_blocks("t.carve", src);
    assert_eq!(errors.len(), 1);
}
