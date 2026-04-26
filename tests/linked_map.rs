use std::collections::BTreeMap;

use linked_map::{linked_map, LinkedMap};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Item {
    id: i32,
    name: &'static str,
}

#[derive(Debug, PartialEq, Eq)]
struct NonClone(&'static str);

#[derive(Debug, PartialEq, Eq)]
struct NonCloneMapped(&'static str);

fn item(id: i32, name: &'static str) -> Item {
    Item { id, name }
}

fn value1() -> Item {
    item(1, "one")
}

fn value2() -> Item {
    item(2, "two")
}

fn value3() -> Item {
    item(3, "three")
}

fn value4() -> Item {
    item(4, "four")
}

fn value5() -> Item {
    item(5, "five")
}

fn empty_linked_map() -> LinkedMap<i32, Item> {
    LinkedMap::new()
}

fn one_linked() -> LinkedMap<i32, Item> {
    linked_map!(1 => value1())
}

fn two_linked() -> LinkedMap<i32, Item> {
    linked_map!(1 => value1(), 2 => value2())
}

fn three_linked() -> LinkedMap<i32, Item> {
    linked_map!(1 => value1(), 2 => value2(), 3 => value3())
}

fn four_linked() -> LinkedMap<i32, Item> {
    linked_map!(1 => value1(), 2 => value2(), 3 => value3(), 4 => value4())
}

#[test]
fn constructor_accepts_entries() {
    let linked = LinkedMap::from_entries([(1, value1())]);
    assert_eq!(linked.len(), 1);
    assert_eq!(linked.get(&1), Some(&value1()));
}

#[test]
fn display_for_empty_map_matches_source_style() {
    assert_eq!(empty_linked_map().to_string(), "LinkedMap []");
}

#[test]
fn push_adds_values_to_the_end() {
    let one = empty_linked_map().push(value1(), 1);
    let two = one.push(value2(), 2);

    assert_eq!(one.len(), 1);
    assert_eq!(two, two_linked());
}

#[test]
fn map_values_preserves_order() {
    let mapped = three_linked().map_values(|_, _| "malcom");
    assert_eq!(
        mapped.to_vec(),
        vec![(1, "malcom"), (2, "malcom"), (3, "malcom")]
    );
}

#[test]
fn get_and_current_value_work() {
    let linked = three_linked();

    assert_eq!(linked.get(&1), Some(&value1()));
    assert_eq!(linked.get(&2), Some(&value2()));
    assert_eq!(linked.current_value(), Some(&value1()));
    assert_eq!(linked.next().current_value(), Some(&value2()));
    assert_eq!(linked.next().next().current_value(), Some(&value3()));
    assert_eq!(linked.get(&99), None);
}

#[test]
fn set_and_update_replace_existing_values() {
    let renamed = three_linked().set(&1, item(1, "renamed"));
    let updated = three_linked().update(&2, |value| item(value.id, "twotwo"));

    assert_eq!(renamed.get(&1), Some(&item(1, "renamed")));
    assert_eq!(updated.get(&2), Some(&item(2, "twotwo")));
}

#[test]
fn movement_updates_cursor() {
    let linked = three_linked();

    assert_eq!(linked.move_to(&2).current_value(), Some(&value2()));
    assert_eq!(linked.move_to_end().current_value(), Some(&value3()));
    assert_eq!(linked.move_to_end().prev().current_value(), Some(&value2()));
    assert_eq!(linked.move_to_start().current_value(), Some(&value1()));
    assert_eq!(linked.move_to(&99).current_value(), Some(&value1()));
}

#[test]
fn inserting_after_cursor_falls_off_preserves_none_cursor() {
    let linked = one_linked().move_to_end().next().push(value2(), 2);

    assert_eq!(linked.current_value(), None);
    assert_eq!(linked.to_vec(), vec![(1, value1()), (2, value2())]);
}

#[test]
fn clear_returns_an_empty_map() {
    assert_eq!(three_linked().clear(), empty_linked_map());
    assert_eq!(empty_linked_map().clear(), empty_linked_map());
}

#[test]
fn equality_ignores_cursor_position() {
    let left = three_linked().move_to(&3);
    let right = three_linked().move_to(&1);

    assert_eq!(left, right);
}

#[test]
fn to_vec_and_copy_preserve_contents() {
    let linked = three_linked();
    let copied = linked.copy();

    assert_eq!(
        linked.to_vec(),
        vec![(1, value1()), (2, value2()), (3, value3())]
    );
    assert_eq!(copied, linked);
}

#[test]
fn reduce_and_reduce_right_follow_iteration_order() {
    let linked = three_linked();

    let reduced = linked.reduce(String::new(), |mut acc, value, key| {
        acc.push_str(&format!("{key}{}", value.name));
        acc
    });

    let reduced_right = linked.reduce_right(String::new(), |mut acc, value, key| {
        acc.push_str(&format!("{key}{}", value.name));
        acc
    });

    assert_eq!(reduced, "1one2two3three");
    assert_eq!(reduced_right, "3three2two1one");
}

#[test]
fn for_each_and_values_iterate_in_order() {
    let linked = two_linked();
    let mut seen = Vec::new();

    linked.for_each(|value, key| seen.push((*key, value.clone())));

    assert_eq!(seen, vec![(1, value1()), (2, value2())]);
    assert_eq!(
        linked.values().cloned().collect::<Vec<_>>(),
        vec![value1(), value2()]
    );
}

#[test]
fn to_map_returns_all_entries() {
    let mut expected = BTreeMap::new();
    expected.insert(1, value1());
    expected.insert(2, value2());
    expected.insert(3, value3());

    assert_eq!(three_linked().to_map(), expected);
}

#[test]
fn remove_and_delete_drop_keys() {
    assert_eq!(three_linked().remove(&3), two_linked());
    assert_eq!(three_linked().delete(&3).delete(&2), one_linked());
    assert_eq!(three_linked().remove(&1).first(), Some(&value2()));
    assert_eq!(three_linked().remove(&3).last(), Some(&value2()));
}

#[test]
fn shift_and_pop_remove_from_ends() {
    assert_eq!(
        three_linked().shift(),
        linked_map!(2 => value2(), 3 => value3())
    );
    assert_eq!(three_linked().pop(), two_linked());
    assert_eq!(three_linked().shift().first(), Some(&value2()));
    assert_eq!(three_linked().pop().last(), Some(&value2()));
}

#[test]
fn prepend_and_unshift_add_to_the_front() {
    assert_eq!(
        one_linked().prepend(value2(), 2),
        linked_map!(2 => value2(), 1 => value1())
    );

    assert_eq!(
        one_linked().unshift([(2, value2()), (3, value3())]),
        linked_map!(2 => value2(), 3 => value3(), 1 => value1())
    );
}

#[test]
fn insert_after_and_before_place_values_relative_to_existing_keys() {
    assert_eq!(
        two_linked().insert_after(&1, value3(), 4),
        linked_map!(1 => value1(), 4 => value3(), 2 => value2())
    );

    assert_eq!(
        two_linked().insert_before(&1, value3(), 4),
        linked_map!(4 => value3(), 1 => value1(), 2 => value2())
    );
}

#[test]
fn swap_and_reverse_reorder_without_changing_values() {
    assert_eq!(
        two_linked().swap(&1, &2),
        linked_map!(2 => value2(), 1 => value1())
    );

    assert_eq!(
        four_linked().swap(&1, &3),
        linked_map!(3 => value3(), 2 => value2(), 1 => value1(), 4 => value4())
    );

    assert_eq!(
        three_linked().reverse(),
        linked_map!(3 => value3(), 2 => value2(), 1 => value1())
    );
}

#[test]
fn get_between_respects_inclusion_flags() {
    assert_eq!(
        four_linked().get_between(&1, &4, false, false),
        linked_map!(2 => value2(), 3 => value3())
    );

    assert_eq!(
        four_linked().get_between(&1, &4, true, false),
        linked_map!(1 => value1(), 2 => value2(), 3 => value3())
    );

    assert_eq!(four_linked().get_between(&1, &4, true, true), four_linked());
}

#[test]
fn get_between_with_same_key_matches_source_behavior() {
    assert_eq!(
        three_linked().get_between(&2, &2, true, false),
        linked_map!(2 => value2())
    );
}

#[test]
fn get_after_and_get_before_follow_order() {
    let linked = four_linked();

    assert_eq!(linked.get_after(&1), Some(&value2()));
    assert_eq!(linked.get_after(&4), None);
    assert_eq!(linked.get_before(&1), None);
    assert_eq!(linked.get_before(&4), Some(&value3()));
}

#[test]
fn delete_between_respects_inclusion_flags() {
    assert_eq!(
        four_linked().delete_between(&1, &4, false, false),
        linked_map!(1 => value1(), 4 => value4())
    );

    assert_eq!(
        four_linked().delete_between(&1, &4, true, false),
        linked_map!(4 => value4())
    );

    assert_eq!(
        four_linked().delete_between(&1, &4, true, true),
        empty_linked_map()
    );
}

#[test]
fn delete_between_with_same_key_matches_source_behavior() {
    assert_eq!(
        three_linked().delete_between(&2, &2, true, false),
        linked_map!(1 => value1(), 3 => value3())
    );
}

#[test]
fn push_many_concat_pop_many_and_insert_many_after_work_together() {
    assert_eq!(
        four_linked()
            .get_between(&2, &4, false, false)
            .push_many([(1, value1()), (2, value2())]),
        linked_map!(3 => value3(), 1 => value1(), 2 => value2())
    );

    assert_eq!(
        two_linked().concat(&linked_map!(3 => value3(), 4 => value4())),
        four_linked()
    );

    assert_eq!(four_linked().pop_many(1), three_linked());
    assert_eq!(four_linked().pop_many(2), two_linked());
    assert_eq!(four_linked().pop_many(3), one_linked());

    assert_eq!(
        linked_map!(1 => value1())
            .insert_many_after(&1, [(2, value2()), (3, value3()), (4, value4())]),
        four_linked()
    );
}

#[test]
fn iter_supports_double_ended_iteration() {
    let linked = three_linked();
    let mut iter = linked.iter();

    assert_eq!(iter.next(), Some((&1, &value1())));
    assert_eq!(iter.next_back(), Some((&3, &value3())));
    assert_eq!(iter.next(), Some((&2, &value2())));
    assert_eq!(iter.next_back(), None);
}

#[test]
fn iter_and_values_report_exact_remaining_len() {
    let linked = three_linked();

    let mut iter = linked.iter();
    assert_eq!(iter.len(), 3);
    iter.next();
    assert_eq!(iter.len(), 2);

    let mut values = linked.values();
    assert_eq!(values.len(), 3);
    values.next();
    assert_eq!(values.len(), 2);
}

#[test]
fn read_only_access_works_with_non_clone_values() {
    let linked = LinkedMap::from_entries([(1, NonClone("one"))]);

    assert_eq!(linked.len(), 1);
    assert_eq!(linked.get(&1), Some(&NonClone("one")));
    assert_eq!(linked.current_value(), Some(&NonClone("one")));
    assert_eq!(
        linked
            .iter()
            .map(|(key, value)| (*key, value.0))
            .collect::<Vec<_>>(),
        vec![(1, "one")]
    );
}

#[test]
fn cursor_movement_and_reductions_work_with_non_clone_values() {
    let linked = LinkedMap::from_entries([(1, NonClone("one")), (2, NonClone("two"))]);

    assert_eq!(linked.move_to_end().current_value(), Some(&NonClone("two")));
    assert_eq!(
        linked.move_to_end().prev().current_value(),
        Some(&NonClone("one"))
    );

    let reduced = linked.reduce(String::new(), |mut acc, value, key| {
        acc.push_str(&format!("{key}{}", value.0));
        acc
    });
    let reduced_right = linked.reduce_right(String::new(), |mut acc, value, key| {
        acc.push_str(&format!("{key}{}", value.0));
        acc
    });

    let mut seen = Vec::new();
    linked.for_each(|value, key| seen.push((*key, value.0)));

    assert_eq!(reduced, "1one2two");
    assert_eq!(reduced_right, "2two1one");
    assert_eq!(seen, vec![(1, "one"), (2, "two")]);
}

#[test]
fn map_values_accepts_non_clone_input_and_output_types() {
    let linked = LinkedMap::from_entries([(1, NonClone("one")), (2, NonClone("two"))]);
    let mapped = linked.map_values(|value, _| NonCloneMapped(value.0));

    assert_eq!(mapped.get(&1), Some(&NonCloneMapped("one")));
    assert_eq!(mapped.get(&2), Some(&NonCloneMapped("two")));
}

#[test]
fn extra_insertions_match_the_source_examples() {
    let linked = three_linked().insert_after(&3, value5(), 5);
    assert_eq!(linked.last(), Some(&value5()));

    let linked = three_linked().insert_before(&1, value5(), 5);
    assert_eq!(linked.first(), Some(&value5()));
}
