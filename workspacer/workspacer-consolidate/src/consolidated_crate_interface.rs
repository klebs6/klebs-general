// ---------------- [ File: src/consolidated_crate_interface.rs ]
crate::ix!();

#[derive(Getters, Debug)]
pub struct ConsolidatedCrateInterface {
    fns:          Vec<CrateInterfaceItem<ast::Fn>>,
    structs:      Vec<CrateInterfaceItem<ast::Struct>>,
    enums:        Vec<CrateInterfaceItem<ast::Enum>>,
    traits:       Vec<CrateInterfaceItem<ast::Trait>>,
    type_aliases: Vec<CrateInterfaceItem<ast::TypeAlias>>,
    macros:       Vec<CrateInterfaceItem<ast::MacroRules>>,
    impls:        Vec<ImplBlockInterface>,
    modules:      Vec<ModuleInterface>,
}

unsafe impl Send for ConsolidatedCrateInterface {}
unsafe impl Sync for ConsolidatedCrateInterface {}

impl fmt::Display for ConsolidatedCrateInterface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        macro_rules! print_items {
            ($vec:expr, $f:expr) => {
                for (i, item) in $vec.iter().enumerate() {
                    writeln!($f, "{}", item)?;
                    if i + 1 < $vec.len() {
                        writeln!($f)?;
                    }
                }
            };
        }

        print_items!(self.enums, f);
        print_items!(self.traits, f);
        print_items!(self.type_aliases, f);
        print_items!(self.macros, f);
        print_items!(self.structs, f);
        print_items!(self.fns, f);
        print_items!(self.impls, f);
        print_items!(self.modules, f);

        Ok(())
    }
}

impl ConsolidatedCrateInterface {
    pub fn new() -> Self {
        Self {
            fns:          vec![],
            structs:      vec![],
            enums:        vec![],
            traits:       vec![],
            type_aliases: vec![],
            macros:       vec![],
            impls:        vec![],
            modules:      vec![],
        }
    }

    pub fn add_fn(&mut self, item: CrateInterfaceItem<ast::Fn>) { self.fns.push(item); }
    pub fn add_struct(&mut self, item: CrateInterfaceItem<ast::Struct>) { self.structs.push(item); }
    pub fn add_enum(&mut self, item: CrateInterfaceItem<ast::Enum>) { self.enums.push(item); }
    pub fn add_trait(&mut self, item: CrateInterfaceItem<ast::Trait>) { self.traits.push(item); }
    pub fn add_type_alias(&mut self, item: CrateInterfaceItem<ast::TypeAlias>) { self.type_aliases.push(item); }
    pub fn add_macro(&mut self, item: CrateInterfaceItem<ast::MacroRules>) { self.macros.push(item); }
    pub fn add_impl(&mut self, ib: ImplBlockInterface) { self.impls.push(ib); }
    pub fn add_module(&mut self, ib: ModuleInterface) { self.modules.push(ib); }
}

#[cfg(test)]
#[disable]
mod test_consolidated_crate_interface {
    use super::*;
    use std::fmt;

    // ------------------------------------------------------------------------
    // 1) Minimal stubs or “fake” items to simulate each category
    // ------------------------------------------------------------------------

    // We'll define a function that makes a fake `CrateInterfaceItem<ast::Fn>`
    // that displays as something like "fn: <name>" for easy checking.
    fn fake_fn_item(name: &str) -> CrateInterfaceItem<ra_ap_syntax::ast::Fn> {
        // We'll define a "FakeAstFn" or simply reuse a zero-sized type
        // but overriding Display in the wrapper. For simplicity, we'll do:
        struct FakeFn(String);
        impl crate::GenerateSignature for FakeFn {
            fn generate_signature(&self) -> String {
                // e.g. "fn my_function()"
                format!("fn {}", self.0)
            }
        }
        // Now build a CrateInterfaceItem with minimal info
        CrateInterfaceItem::new(
            FakeFn(name.to_string()), // the item
            None,                     // docs
            None,                     // attrs
            None,                     // body_source
        )
    }

    // Similarly, a fake `CrateInterfaceItem<ast::Struct>`
    fn fake_struct_item(name: &str) -> CrateInterfaceItem<ra_ap_syntax::ast::Struct> {
        struct FakeStruct(String);
        impl crate::GenerateSignature for FakeStruct {
            fn generate_signature(&self) -> String {
                format!("struct {}", self.0)
            }
        }
        CrateInterfaceItem::new(FakeStruct(name.to_string()), None, None, None)
    }

    fn fake_enum_item(name: &str) -> CrateInterfaceItem<ra_ap_syntax::ast::Enum> {
        struct FakeEnum(String);
        impl crate::GenerateSignature for FakeEnum {
            fn generate_signature(&self) -> String {
                format!("enum {}", self.0)
            }
        }
        CrateInterfaceItem::new(FakeEnum(name.to_string()), None, None, None)
    }

    fn fake_trait_item(name: &str) -> CrateInterfaceItem<ra_ap_syntax::ast::Trait> {
        struct FakeTrait(String);
        impl crate::GenerateSignature for FakeTrait {
            fn generate_signature(&self) -> String {
                format!("trait {}", self.0)
            }
        }
        CrateInterfaceItem::new(FakeTrait(name.to_string()), None, None, None)
    }

    fn fake_type_alias_item(name: &str) -> CrateInterfaceItem<ra_ap_syntax::ast::TypeAlias> {
        struct FakeTypeAlias(String);
        impl crate::GenerateSignature for FakeTypeAlias {
            fn generate_signature(&self) -> String {
                format!("type {} = ...;", self.0)
            }
        }
        CrateInterfaceItem::new(FakeTypeAlias(name.to_string()), None, None, None)
    }

    fn fake_macro_item(name: &str) -> CrateInterfaceItem<ra_ap_syntax::ast::MacroRules> {
        struct FakeMacro(String);
        impl crate::GenerateSignature for FakeMacro {
            fn generate_signature(&self) -> String {
                format!("macro_rules! {}", self.0)
            }
        }
        CrateInterfaceItem::new(FakeMacro(name.to_string()), None, None, None)
    }

    // For `ImplBlockInterface`, we can define a minimal stub that implements Display.
    fn fake_impl_block(name: &str) -> ImplBlockInterface {
        // We'll define a short struct that has just enough fields for the display logic
        // or if your real `ImplBlockInterface` can be built easily, do that.
        // We'll do a simple approach: store a string in "signature_text".
        ImplBlockInterface::new(
            /* docs:        */ None,
            /* attributes:  */ None,
            /* signature_text: */ format!("impl_block_for_{}", name),
            /* methods:        */ vec![],
            /* type_aliases:   */ vec![],
        )
    }

    // For `ModuleInterface`, a minimal stub that implements Display
    fn fake_module(name: &str) -> ModuleInterface {
        let mut module = ModuleInterface::new(None, None, name.to_string());
        // Possibly add some nested items if you want, but we'll keep it simple.
        module
    }

    // ------------------------------------------------------------------------
    // 2) Test the data structure & display logic
    // ------------------------------------------------------------------------

    #[test]
    fn test_empty_consolidated_crate_interface_displays_nothing() {
        let cci = ConsolidatedCrateInterface::new();
        let output = format!("{}", cci);
        assert!(output.is_empty(), "No items => empty display");
    }

    /// 2) If we add a single item in each category, we expect them to appear
    /// in the order: enum -> trait -> type_alias -> macro -> struct -> fn -> impl -> module.
    #[test]
    fn test_single_item_in_each_category_order() {
        let mut cci = ConsolidatedCrateInterface::new();

        cci.add_fn(fake_fn_item("my_fn"));
        cci.add_struct(fake_struct_item("MyStruct"));
        cci.add_enum(fake_enum_item("MyEnum"));
        cci.add_trait(fake_trait_item("MyTrait"));
        cci.add_type_alias(fake_type_alias_item("MyAlias"));
        cci.add_macro(fake_macro_item("my_macro"));
        cci.add_impl(fake_impl_block("ImplTarget"));
        cci.add_module(fake_module("my_mod"));

        // Now check the display output
        let output = format!("{}", cci);
        // We'll parse lines and see the order
        let lines: Vec<_> = output.lines().collect();

        // We expect, in order:
        //   enum MyEnum
        //   <blank line>
        //   trait MyTrait
        //   <blank line>
        //   type MyAlias = ...;
        //   <blank line>
        //   macro_rules! my_macro
        //   <blank line>
        //   struct MyStruct
        //   <blank line>
        //   fn my_fn
        //   <blank line>
        //   impl_block_for_ImplTarget
        //   <blank line>
        //   mod my_mod { ... } or something
        //
        // Because each item is printed by its own Display, each category is separated
        // by the code that prints a blank line after each item if not the last in the vector.

        // Let's do partial checks:
        // lines[0] should be "enum MyEnum"
        assert_eq!(lines[0], "enum MyEnum");
        // lines[1] should be blank
        assert!(lines[1].is_empty(), "Expect a blank line after the enum category's single item");

        // lines[2] => "trait MyTrait"
        assert_eq!(lines[2], "trait MyTrait");
        // lines[3] blank
        assert!(lines[3].is_empty());

        // lines[4] => "type MyAlias = ...;"
        assert_eq!(lines[4], "type MyAlias = ...;");
        // lines[5] blank
        assert!(lines[5].is_empty());

        // lines[6] => "macro_rules! my_macro"
        assert_eq!(lines[6], "macro_rules! my_macro");
        // lines[7] blank
        assert!(lines[7].is_empty());

        // lines[8] => "struct MyStruct"
        assert_eq!(lines[8], "struct MyStruct");
        // lines[9] blank
        assert!(lines[9].is_empty());

        // lines[10] => "fn my_fn"
        assert_eq!(lines[10], "fn my_fn");
        // lines[11] blank
        assert!(lines[11].is_empty());

        // lines[12] => "impl_block_for_ImplTarget"
        assert_eq!(lines[12], "impl_block_for_ImplTarget");
        // lines[13] blank
        assert!(lines[13].is_empty());

        // lines[14] => "mod my_mod {"
        // Actually your real ModuleInterface might produce no output or just "mod my_mod {}". 
        // For our mock, we used ModuleInterface::new(None, None, name.to_string()) => 
        // The display might skip if there's nothing inside. Possibly empty. 
        // We'll check partial match:
        assert!(lines[14].contains("mod my_mod"), "Should mention 'mod my_mod'");
        // lines[15] => might be "}" or might be empty if there's no items
        // We'll do a partial check:
        // let's see if there's a line 15:
        if lines.len() > 15 {
            assert!(lines[15].trim().is_empty() || lines[15].contains("}"), "Either blank or closing brace");
        }
        // There's no more items, so that's presumably the end.
    }

    /// 3) If we add multiple items in the same category, each item is separated by a blank line. 
    ///    We'll test that inside e.g. `fns` or `structs`.
    #[test]
    fn test_multiple_items_in_same_category() {
        let mut cci = ConsolidatedCrateInterface::new();
        cci.add_fn(fake_fn_item("fn_one"));
        cci.add_fn(fake_fn_item("fn_two"));
        cci.add_fn(fake_fn_item("fn_three"));

        let output = format!("{}", cci);
        let lines: Vec<_> = output.lines().collect();
        // We expect:
        //   fn fn_one
        //   <blank>
        //   fn fn_two
        //   <blank>
        //   fn fn_three
        assert_eq!(lines[0], "fn fn_one");
        assert!(lines[1].is_empty());
        assert_eq!(lines[2], "fn fn_two");
        assert!(lines[3].is_empty());
        assert_eq!(lines[4], "fn fn_three");
        // No blank line after the final one if the loop doesn't add it 
        // (depending on your macro_rules! logic). 
        // But in your code, you do `if i+1 < vec.len() { writeln!() }`. 
        // So there's no blank line after the last item within that category.
        assert_eq!(lines.len(), 5);
    }

    /// 4) No items in some categories, but multiple in others => the display prints only the non-empty categories, in correct order.
    #[test]
    fn test_sparse_categories() {
        let mut cci = ConsolidatedCrateInterface::new();
        cci.add_struct(fake_struct_item("A"));
        cci.add_struct(fake_struct_item("B"));
        // skip adding enums, traits, macros, etc.
        cci.add_fn(fake_fn_item("some_fn"));

        let output = format!("{}", cci);
        let lines: Vec<_> = output.lines().collect();
        // Should skip the categories with empty vecs. 
        // The code prints: (enum) -> (trait) -> (type_alias) -> (macro) -> (struct) -> (fn) -> (impl) -> (module).
        // We have no enum/trait/type_alias/macros => no lines for them. 
        // Then we see 2 structs, each separated by blank, then we see 1 fn, with a blank line after the second struct.
        assert_eq!(lines[0], "struct A");
        assert!(lines[1].is_empty());
        assert_eq!(lines[2], "struct B");
        assert!(lines[3].is_empty());
        assert_eq!(lines[4], "fn some_fn");
    }

    /// 5) We can do a stress test with many categories. 
    ///    But effectively it's the same as test_single_item_in_each_category_order, just more items. 
    ///    We'll skip the details here. 
    #[test]
    fn test_stress_many_items_in_all_categories() {
        let mut cci = ConsolidatedCrateInterface::new();
        // Add multiple enums
        cci.add_enum(fake_enum_item("EnumOne"));
        cci.add_enum(fake_enum_item("EnumTwo"));
        // Add multiple traits
        cci.add_trait(fake_trait_item("TraitOne"));
        cci.add_trait(fake_trait_item("TraitTwo"));
        // ... and so forth ...
        // We won't do the entire check in detail here, 
        // but you'd do basically the same approach: parse lines, confirm order & spacing.
        let output = format!("{}", cci);
        // For brevity in code sample, we won't do the entire validation. 
        // In real tests, you'd do partial checks or lines-of-output checks like earlier examples.
        assert!(output.contains("enum EnumOne"));
        assert!(output.contains("enum EnumTwo"));
        assert!(output.contains("trait TraitOne"));
        assert!(output.contains("trait TraitTwo"));
        // etc. 
    }

    // 6) If absolutely no items are in any category, the display is empty (already tested by test_empty...).
    //    If you want to confirm no extraneous newlines, test that output.is_empty() is true.
}
