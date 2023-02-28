use std::any::Any;
use std::fmt::{Debug, Display};
use std::ops::RangeInclusive;
use typed_builder::TypedBuilder;

pub use crate::{ExtcapFormatter, PrintConfig};

macro_rules! generate_config_ext {
    ($config_type:ty) => {
        impl ConfigExtGenerated for $config_type {
            fn call(&self) -> &str {
                &self.call
            }

            fn as_any(&self) -> &dyn Any {
                self
            }
        }
    };
}

/// A functional trait for [`SelectorConfig::reload`]. Users normally do not
/// have to use this trait directly, as it is automatically implemented for all
/// `Fn() -> Vec<ConfigOptionValue> + Sync + 'static`, so callers can simply
/// pass a closure into `reload()`.
pub trait ReloadFn: Fn() -> Vec<ConfigOptionValue> + Sync + 'static {}

impl<F> ReloadFn for F where F: Fn() -> Vec<ConfigOptionValue> + Sync + 'static {}

impl std::fmt::Debug for dyn ReloadFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ReloadFn")
    }
}

/// Option fields where the user may choose from one or more options. If parent
/// is provided for the value items, the option fields for multicheck and
/// selector are presented in a tree-like structure. selector and radio values
/// must present a default value, which will be the value provided to the extcap
/// binary for this argument. editselector option fields let the user select
/// from a list of items or enter a custom value.
///
/// ```
/// use rust_extcap::config::*;
///
/// let selector = SelectorConfig::builder()
///     .config_number(3)
///     .call("remote")
///     .display("Remote Channel")
///     .tooltip("Remote Channel Selector")
///     .options([
///         ConfigOptionValue::builder().value("if1").display("Remote1").default(true).build(),
///         ConfigOptionValue::builder().value("if2").display("Remote2").build(),
///     ])
///     .build();
/// assert_eq!(
///     format!("{}", ExtcapFormatter(&selector)),
///     concat!(
///         "arg {number=3}{call=--remote}{display=Remote Channel}{tooltip=Remote Channel Selector}{type=selector}\n",
///         "value {arg=3}{value=if1}{display=Remote1}{default=true}\n",
///         "value {arg=3}{value=if2}{display=Remote2}{default=false}\n"
///     )
/// );
/// ```
#[derive(Debug, TypedBuilder)]
pub struct SelectorConfig {
    pub config_number: u8,
    #[builder(setter(into))]
    pub call: String,
    #[builder(setter(into))]
    pub display: String,
    #[builder(default, setter(strip_option, into))]
    pub tooltip: Option<String>,
    #[builder(default, setter(strip_option))]
    pub reload: Option<Box<dyn ReloadFn>>,
    #[builder(default, setter(strip_option, into))]
    pub placeholder: Option<String>,
    #[builder(setter(into))]
    pub options: Vec<ConfigOptionValue>,
}

impl ConfigTrait for SelectorConfig {}
impl<'a> Display for ExtcapFormatter<&'a SelectorConfig> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "arg {{number={}}}", self.0.config_number)?;
        write!(f, "{{call=--{}}}", self.0.call)?;
        write!(f, "{{display={}}}", self.0.display)?;
        if let Some(tooltip) = &self.0.tooltip {
            write!(f, "{{tooltip={tooltip}}}")?;
        }
        if let Some(placeholder) = &self.0.placeholder {
            write!(f, "{{placeholder={}}}", placeholder)?;
        }
        write!(f, "{{type=selector}}")?;
        if self.0.reload.is_some() {
            write!(f, "{{reload=true}}")?;
        }
        writeln!(f)?;
        for opt in self.0.options.iter() {
            write!(f, "{}", ExtcapFormatter(&(opt, self.0.config_number)))?;
        }
        Ok(())
    }
}

generate_config_ext!(SelectorConfig);

/// Option fields where the user may choose from one or more options. If parent
/// is provided for the value items, the option fields for multicheck and
/// selector are presented in a tree-like structure. selector and radio values
/// must present a default value, which will be the value provided to the extcap
/// binary for this argument. editselector option fields let the user select
/// from a list of items or enter a custom value.
///
/// ```
/// use rust_extcap::config::*;
///
/// let edit_selector = EditSelectorConfig::builder()
///     .config_number(3)
///     .call("remote")
///     .display("Remote Channel")
///     .tooltip("Remote Channel Selector")
///     .options([
///         ConfigOptionValue::builder().value("if1").display("Remote1").default(true).build(),
///         ConfigOptionValue::builder().value("if2").display("Remote2").build(),
///     ])
///     .build();
/// assert_eq!(
///     format!("{}", ExtcapFormatter(&edit_selector)),
///     concat!(
///         "arg {number=3}{call=--remote}{display=Remote Channel}{tooltip=Remote Channel Selector}{type=editselector}\n",
///         "value {arg=3}{value=if1}{display=Remote1}{default=true}\n",
///         "value {arg=3}{value=if2}{display=Remote2}{default=false}\n"
///     )
/// );
/// ```
#[derive(Debug, TypedBuilder)]
pub struct EditSelectorConfig {
    pub config_number: u8,
    #[builder(setter(into))]
    pub call: String,
    #[builder(setter(into))]
    pub display: String,
    #[builder(default, setter(strip_option, into))]
    pub tooltip: Option<String>,
    #[builder(default)]
    pub reload: Option<Box<dyn ReloadFn>>,
    #[builder(default, setter(strip_option, into))]
    pub placeholder: Option<String>,
    #[builder(setter(into))]
    pub options: Vec<ConfigOptionValue>,
}

impl ConfigTrait for EditSelectorConfig {}
impl<'a> Display for ExtcapFormatter<&'a EditSelectorConfig> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "arg {{number={}}}", self.0.config_number)?;
        write!(f, "{{call=--{}}}", self.0.call)?;
        write!(f, "{{display={}}}", self.0.display)?;
        if let Some(tooltip) = &self.0.tooltip {
            write!(f, "{{tooltip={tooltip}}}")?;
        }
        if let Some(placeholder) = &self.0.placeholder {
            write!(f, "{{placeholder={}}}", placeholder)?;
        }
        write!(f, "{{type=editselector}}")?;
        if self.0.reload.is_some() {
            write!(f, "{{reload=true}}")?;
        }
        writeln!(f)?;
        for opt in self.0.options.iter() {
            write!(f, "{}", ExtcapFormatter(&(opt, self.0.config_number)))?;
        }
        Ok(())
    }
}

generate_config_ext!(EditSelectorConfig);

/// Option fields where the user may choose from one or more options. If parent
/// is provided for the value items, the option fields for multicheck and
/// selector are presented in a tree-like structure. selector and radio values
/// must present a default value, which will be the value provided to the extcap
/// binary for this argument. editselector option fields let the user select
/// from a list of items or enter a custom value.
///
/// ```
/// use rust_extcap::config::*;
///
/// let radio = RadioConfig::builder()
///     .config_number(3)
///     .call("remote")
///     .display("Remote Channel")
///     .tooltip("Remote Channel Selector")
///     .options([
///         ConfigOptionValue::builder().value("if1").display("Remote1").default(true).build(),
///         ConfigOptionValue::builder().value("if2").display("Remote2").build(),
///     ])
///     .build();
/// assert_eq!(
///     format!("{}", ExtcapFormatter(&radio)),
///     concat!(
///         "arg {number=3}{call=--remote}{display=Remote Channel}{tooltip=Remote Channel Selector}{type=radio}\n",
///         "value {arg=3}{value=if1}{display=Remote1}{default=true}\n",
///         "value {arg=3}{value=if2}{display=Remote2}{default=false}\n"
///     )
/// );
/// ```
#[derive(Debug, TypedBuilder)]
pub struct RadioConfig {
    pub config_number: u8,
    #[builder(setter(into))]
    pub call: String,
    #[builder(setter(into))]
    pub display: String,
    #[builder(default, setter(strip_option, into))]
    pub tooltip: Option<String>,
    #[builder(default, setter(strip_option, into))]
    pub group: Option<String>,
    #[builder(setter(into))]
    pub options: Vec<ConfigOptionValue>,
}

impl ConfigTrait for RadioConfig {}
impl<'a> Display for ExtcapFormatter<&'a RadioConfig> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "arg {{number={}}}", self.0.config_number)?;
        write!(f, "{{call=--{}}}", self.0.call)?;
        write!(f, "{{display={}}}", self.0.display)?;
        if let Some(tooltip) = &self.0.tooltip {
            write!(f, "{{tooltip={tooltip}}}")?;
        }
        if let Some(group) = &self.0.group {
            write!(f, "{{group={}}}", group)?;
        }
        write!(f, "{{type=radio}}")?;
        writeln!(f)?;
        for opt in self.0.options.iter() {
            write!(f, "{}", ExtcapFormatter(&(opt, self.0.config_number)))?;
        }
        Ok(())
    }
}

generate_config_ext!(RadioConfig);

/// Option fields where the user may choose from one or more options. If parent
/// is provided for the value items, the option fields for multicheck and
/// selector are presented in a tree-like structure. selector and radio values
/// must present a default value, which will be the value provided to the extcap
/// binary for this argument. editselector option fields let the user select
/// from a list of items or enter a custom value.
///
/// ```
/// use rust_extcap::config::*;
///
/// let config = MultiCheckConfig::builder()
///     .config_number(3)
///     .call("multi")
///     .display("Remote Channel")
///     .tooltip("Remote Channel Selector")
///     .options([
///         MultiCheckValue::builder().value("if1").display("Remote1").default_value(true).build(),
///         MultiCheckValue::builder().value("if2").display("Remote2").children([
///             MultiCheckValue::builder().value("if2a").display("Remote2A").default_value(true).build(),
///             MultiCheckValue::builder().value("if2b").display("Remote2B").default_value(true).build(),
///         ]).build(),
///     ])
///     .build();
/// assert_eq!(
///     format!("{}", ExtcapFormatter(&config)),
///     concat!(
///         "arg {number=3}{call=--multi}{display=Remote Channel}{tooltip=Remote Channel Selector}{type=multicheck}\n",
///         "value {arg=3}{value=if1}{display=Remote1}{default=true}{enabled=true}\n",
///         "value {arg=3}{value=if2}{display=Remote2}{default=false}{enabled=true}\n",
///         "value {arg=3}{value=if2a}{display=Remote2A}{default=true}{enabled=true}{parent=if2}\n",
///         "value {arg=3}{value=if2b}{display=Remote2B}{default=true}{enabled=true}{parent=if2}\n"
///     )
/// );
/// ```
#[derive(Debug, TypedBuilder)]
pub struct MultiCheckConfig {
    pub config_number: u8,
    #[builder(setter(into))]
    pub call: String,
    #[builder(setter(into))]
    pub display: String,
    #[builder(default, setter(strip_option, into))]
    pub tooltip: Option<String>,
    #[builder(default, setter(strip_option, into))]
    pub group: Option<String>,
    #[builder(setter(into))]
    pub options: Vec<MultiCheckValue>,
}

impl ConfigTrait for MultiCheckConfig {}
impl<'a> Display for ExtcapFormatter<&'a MultiCheckConfig> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "arg {{number={}}}", self.0.config_number)?;
        write!(f, "{{call=--{}}}", self.0.call)?;
        write!(f, "{{display={}}}", self.0.display)?;
        if let Some(tooltip) = &self.0.tooltip {
            write!(f, "{{tooltip={tooltip}}}")?;
        }
        if let Some(group) = &self.0.group {
            write!(f, "{{group={}}}", group)?;
        }
        write!(f, "{{type=multicheck}}")?;
        writeln!(f)?;
        for opt in self.0.options.iter() {
            write!(f, "{}", ExtcapFormatter((opt, self.0.config_number, None)))?;
        }
        Ok(())
    }
}

generate_config_ext!(MultiCheckConfig);

/// Represents a checkbox in a [`MultiCheckConfig`]. Each value is a checkbox in
/// the UI that can be nested into a hierarchy using the `children` field. See
/// the docs for [`MultiCheckConfig`] for usage details.
#[derive(Debug, Clone, TypedBuilder)]
pub struct MultiCheckValue {
    #[builder(setter(into))]
    value: String,
    #[builder(setter(into))]
    display: String,
    #[builder(default = false)]
    default_value: bool,
    #[builder(default = true)]
    enabled: bool,
    #[builder(default, setter(into))]
    children: Vec<MultiCheckValue>,
}

impl<'a> Display for ExtcapFormatter<(&'a MultiCheckValue, u8, Option<&'a MultiCheckValue>)> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let (config, config_number, parent) = self.0;
        write!(f, "value {{arg={}}}", config_number)?;
        write!(f, "{{value={}}}", config.value)?;
        write!(f, "{{display={}}}", config.display)?;
        write!(f, "{{default={}}}", config.default_value)?;
        write!(f, "{{enabled={}}}", config.enabled)?;
        if let Some(parent) = parent {
            write!(f, "{{parent={}}}", parent.value)?;
        }
        writeln!(f)?;
        for c in config.children.iter() {
            write!(f, "{}", Self((c, config_number, Some(config))))?;
        }
        Ok(())
    }
}

/// This provides a field for entering a numeric value of the given data type. A
/// default value may be provided, as well as a range.
///
/// ```
/// use rust_extcap::config::*;
///
/// let config = LongConfig::builder()
///     .config_number(0)
///     .call("delay")
///     .display("Time delay")
///     .tooltip("Time delay between packages")
///     .range(-2..=15)
///     .default_value(0)
///     .build();
/// assert_eq!(
///     format!("{}", ExtcapFormatter(&config)),
///     "arg {number=0}{call=--delay}{display=Time delay}{tooltip=Time delay between packages}{range=-2,15}{default=0}{type=long}\n"
/// );
/// ```
#[derive(Debug, TypedBuilder)]
pub struct LongConfig {
    pub config_number: u8,
    #[builder(setter(into))]
    pub call: String,
    #[builder(setter(into))]
    pub display: String,
    #[builder(default, setter(strip_option, into))]
    pub tooltip: Option<String>,
    #[builder(default, setter(strip_option))]
    pub range: Option<RangeInclusive<i64>>,
    pub default_value: i64,
    #[builder(default, setter(strip_option, into))]
    pub group: Option<String>,
}

impl ConfigTrait for LongConfig {}
impl<'a> Display for ExtcapFormatter<&'a LongConfig> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "arg {{number={}}}", self.0.config_number)?;
        write!(f, "{{call=--{}}}", self.0.call)?;
        write!(f, "{{display={}}}", self.0.display)?;
        if let Some(tooltip) = &self.0.tooltip {
            write!(f, "{{tooltip={tooltip}}}")?;
        }
        if let Some(range) = &self.0.range {
            write!(f, "{{range={},{}}}", range.start(), range.end())?;
        }
        write!(f, "{{default={}}}", self.0.default_value)?;
        write!(f, "{{type=long}}")?;
        if let Some(group) = &self.0.group {
            write!(f, "{{group={}}}", group)?;
        }
        writeln!(f)?;
        Ok(())
    }
}

generate_config_ext!(LongConfig);

/// This provides a field for entering a numeric value of the given data type. A
/// default value may be provided, as well as a range.
///
/// ```
/// use rust_extcap::config::*;
///
/// let config = IntegerConfig::builder()
///     .config_number(0)
///     .call("delay")
///     .display("Time delay")
///     .tooltip("Time delay between packages")
///     .range(-10..=15)
///     .default_value(0)
///     .build();
/// assert_eq!(
///     format!("{}", ExtcapFormatter(&config)),
///     "arg {number=0}{call=--delay}{display=Time delay}{tooltip=Time delay between packages}{range=-10,15}{default=0}{type=integer}\n"
/// );
/// ```
#[derive(Debug, TypedBuilder)]
pub struct IntegerConfig {
    pub config_number: u8,
    #[builder(setter(into))]
    pub call: String,
    #[builder(setter(into))]
    pub display: String,
    #[builder(default, setter(strip_option, into))]
    pub tooltip: Option<String>,
    #[builder(default, setter(strip_option))]
    pub range: Option<RangeInclusive<i32>>,
    pub default_value: i32,
}

impl ConfigTrait for IntegerConfig {}
impl<'a> Display for ExtcapFormatter<&'a IntegerConfig> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "arg {{number={}}}", self.0.config_number)?;
        write!(f, "{{call=--{}}}", self.0.call)?;
        write!(f, "{{display={}}}", self.0.display)?;
        if let Some(tooltip) = &self.0.tooltip {
            write!(f, "{{tooltip={tooltip}}}")?;
        }
        if let Some(range) = &self.0.range {
            write!(f, "{{range={},{}}}", range.start(), range.end())?;
        }
        write!(f, "{{default={}}}", self.0.default_value)?;
        write!(f, "{{type=integer}}")?;
        writeln!(f)?;
        Ok(())
    }
}

generate_config_ext!(IntegerConfig);

/// This provides a field for entering a numeric value of the given data type. A
/// default value may be provided, as well as a range.
///
/// ```
/// use rust_extcap::config::*;
///
/// let config = UnsignedConfig::builder()
///     .config_number(0)
///     .call("delay")
///     .display("Time delay")
///     .tooltip("Time delay between packages")
///     .range(1..=15)
///     .default_value(0)
///     .build();
/// assert_eq!(
///     format!("{}", ExtcapFormatter(&config)),
///     "arg {number=0}{call=--delay}{display=Time delay}{tooltip=Time delay between packages}{range=1,15}{default=0}{type=unsigned}\n"
/// );
/// ```
#[derive(Debug, TypedBuilder)]
pub struct UnsignedConfig {
    pub config_number: u8,
    #[builder(setter(into))]
    pub call: String,
    #[builder(setter(into))]
    pub display: String,
    #[builder(default, setter(strip_option, into))]
    pub tooltip: Option<String>,
    #[builder(default, setter(strip_option, into))]
    pub range: Option<RangeInclusive<u32>>,
    pub default_value: u32,
}

impl ConfigTrait for UnsignedConfig {}
impl<'a> Display for ExtcapFormatter<&'a UnsignedConfig> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "arg {{number={}}}", self.0.config_number)?;
        write!(f, "{{call=--{}}}", self.0.call)?;
        write!(f, "{{display={}}}", self.0.display)?;
        if let Some(tooltip) = &self.0.tooltip {
            write!(f, "{{tooltip={tooltip}}}")?;
        }
        if let Some(range) = &self.0.range {
            write!(f, "{{range={},{}}}", range.start(), range.end())?;
        }
        write!(f, "{{default={}}}", self.0.default_value)?;
        write!(f, "{{type=unsigned}}")?;
        writeln!(f)?;
        Ok(())
    }
}

generate_config_ext!(UnsignedConfig);

/// This provides a field for entering a numeric value of the given data type. A
/// default value may be provided, as well as a range.
///
/// ```
/// use rust_extcap::config::*;
///
/// let config = DoubleConfig::builder()
///     .config_number(0)
///     .call("delay")
///     .display("Time delay")
///     .tooltip("Time delay between packages")
///     .range(-2.6..=8.2)
///     .default_value(3.3)
///     .build();
/// assert_eq!(
///     format!("{}", ExtcapFormatter(&config)),
///     "arg {number=0}{call=--delay}{display=Time delay}{tooltip=Time delay between packages}{range=-2.6,8.2}{default=3.3}{type=double}\n"
/// );
/// ```
#[derive(Debug, TypedBuilder)]
pub struct DoubleConfig {
    pub config_number: u8,
    #[builder(setter(into))]
    pub call: String,
    #[builder(setter(into))]
    pub display: String,
    #[builder(default, setter(strip_option, into))]
    pub tooltip: Option<String>,
    #[builder(default, setter(strip_option))]
    pub range: Option<RangeInclusive<f64>>,
    pub default_value: f64,
    #[builder(default, setter(strip_option, into))]
    pub group: Option<String>,
}

impl ConfigTrait for DoubleConfig {}
impl<'a> Display for ExtcapFormatter<&'a DoubleConfig> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "arg {{number={}}}", self.0.config_number)?;
        write!(f, "{{call=--{}}}", self.0.call)?;
        write!(f, "{{display={}}}", self.0.display)?;
        if let Some(tooltip) = &self.0.tooltip {
            write!(f, "{{tooltip={tooltip}}}")?;
        }
        if let Some(range) = &self.0.range {
            write!(f, "{{range={},{}}}", range.start(), range.end())?;
        }
        write!(f, "{{default={}}}", self.0.default_value)?;
        write!(f, "{{type=double}}")?;
        if let Some(group) = &self.0.group {
            write!(f, "{{group={}}}", group)?;
        }
        writeln!(f)?;
        Ok(())
    }
}

generate_config_ext!(DoubleConfig);

/// This provides a field for entering a text value.
///
/// ```
/// use rust_extcap::config::*;
///
/// let config = StringConfig::builder()
///     .config_number(1)
///     .call("server")
///     .display("IP Address")
///     .tooltip("IP Address for log server")
///     .validation(r"\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b")
///     .build();
/// assert_eq!(
///     format!("{}", ExtcapFormatter(&config)),
///     concat!(
///         r"arg {number=1}{call=--server}{display=IP Address}{tooltip=IP Address for log server}{validation=\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b}{type=string}",
///         "\n"
///     )
/// );
/// ```
#[derive(Debug, TypedBuilder)]
pub struct StringConfig {
    pub config_number: u8,
    #[builder(setter(into))]
    pub call: String,
    #[builder(setter(into))]
    pub display: String,
    #[builder(default, setter(strip_option, into))]
    pub tooltip: Option<String>,
    #[builder(default, setter(strip_option, into))]
    pub placeholder: Option<String>,
    #[builder(default = false)]
    pub required: bool,

    /// Allows to provide a regular expression string, which is used to check
    /// the user input for validity beyond normal data type or range checks.
    /// Despite what the Wireshark documentation says, back-slashes in this
    /// string do not need to be escaped. Just remember to use a Rust raw string
    /// (e.g. `r"\d\d\d\d"`).
    #[builder(default, setter(strip_option, into))]
    pub validation: Option<String>,
    #[builder(default = false)]
    pub save: bool,
}

impl ConfigTrait for StringConfig {}
impl<'a> Display for ExtcapFormatter<&'a StringConfig> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "arg {{number={}}}", self.0.config_number)?;
        write!(f, "{{call=--{}}}", self.0.call)?;
        write!(f, "{{display={}}}", self.0.display)?;
        if let Some(tooltip) = &self.0.tooltip {
            write!(f, "{{tooltip={tooltip}}}")?;
        }
        if let Some(placeholder) = &self.0.placeholder {
            write!(f, "{{placeholder={}}}", placeholder)?;
        }
        if self.0.required {
            write!(f, "{{required=true}}")?;
        }
        if let Some(validation) = &self.0.validation {
            write!(f, "{{validation={}}}", validation)?;
        }
        if self.0.save {
            write!(f, "{{save=true}}")?;
        }
        write!(f, "{{type=string}}")?;
        writeln!(f)?;
        Ok(())
    }
}

generate_config_ext!(StringConfig);

/// Lets the user provide a masked string to the capture. Password strings are
/// not saved with other capture settings.
///
/// ```
/// use rust_extcap::config::*;
///
/// let config = PasswordConfig::builder()
///     .config_number(0)
///     .call("password")
///     .display("The user password")
///     .tooltip("The password for the connection")
///     .build();
/// assert_eq!(
///     format!("{}", ExtcapFormatter(&config)),
///     "arg {number=0}{call=--password}{display=The user password}{tooltip=The password for the connection}{type=password}\n"
/// );
/// ```
#[derive(Debug, TypedBuilder)]
pub struct PasswordConfig {
    pub config_number: u8,
    #[builder(setter(into))]
    pub call: String,
    #[builder(setter(into))]
    pub display: String,
    #[builder(default, setter(strip_option, into))]
    pub tooltip: Option<String>,
    #[builder(default, setter(strip_option, into))]
    pub placeholder: Option<String>,
    #[builder(default = false)]
    pub required: bool,
    #[builder(default, setter(strip_option, into))]
    pub validation: Option<String>,
}

impl ConfigTrait for PasswordConfig {}
impl<'a> Display for ExtcapFormatter<&'a PasswordConfig> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "arg {{number={}}}", self.0.config_number)?;
        write!(f, "{{call=--{}}}", self.0.call)?;
        write!(f, "{{display={}}}", self.0.display)?;
        if let Some(tooltip) = &self.0.tooltip {
            write!(f, "{{tooltip={tooltip}}}")?;
        }
        if let Some(placeholder) = &self.0.placeholder {
            write!(f, "{{placeholder={}}}", placeholder)?;
        }
        if self.0.required {
            write!(f, "{{required=true}}")?;
        }
        if let Some(validation) = &self.0.validation {
            write!(f, "{{validation={}}}", validation)?;
        }
        write!(f, "{{type=password}}")?;
        writeln!(f)?;
        Ok(())
    }
}

generate_config_ext!(PasswordConfig);

/// A time value displayed as a date/time editor.
///
/// ```
/// use rust_extcap::config::*;
///
/// let config = TimestampConfig::builder()
///     .config_number(9)
///     .call("ts")
///     .display("Start Time")
///     .tooltip("Capture start time")
///     .group("Time / Log")
///     .build();
/// assert_eq!(
///     format!("{}", ExtcapFormatter(&config)),
///     "arg {number=9}{call=--ts}{display=Start Time}{tooltip=Capture start time}{group=Time / Log}{type=timestamp}\n"
/// );
/// ```
#[derive(Debug, TypedBuilder)]
pub struct TimestampConfig {
    pub config_number: u8,
    #[builder(setter(into))]
    pub call: String,
    #[builder(setter(into))]
    pub display: String,
    #[builder(default, setter(strip_option, into))]
    pub tooltip: Option<String>,
    #[builder(setter(into))]
    pub group: String,
}

impl ConfigTrait for TimestampConfig {}
impl<'a> Display for ExtcapFormatter<&'a TimestampConfig> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "arg {{number={}}}", self.0.config_number)?;
        write!(f, "{{call=--{}}}", self.0.call)?;
        write!(f, "{{display={}}}", self.0.display)?;
        if let Some(tooltip) = &self.0.tooltip {
            write!(f, "{{tooltip={tooltip}}}")?;
        }
        write!(f, "{{group={}}}", self.0.group)?;
        write!(f, "{{type=timestamp}}")?;
        writeln!(f)?;
        Ok(())
    }
}

generate_config_ext!(TimestampConfig);

/// Lets the user provide a file path. If mustexist=true is provided, the GUI
/// shows the user a dialog for selecting a file. When mustexist=false is used,
/// the GUI shows the user a file dialog for saving a file.
///
/// ```
/// use rust_extcap::config::*;
///
/// let config = FileSelectConfig::builder()
///     .config_number(3)
///     .call("logfile")
///     .display("Logfile")
///     .tooltip("A file for log messages")
///     .must_exist(false)
///     .build();
/// assert_eq!(
///     format!("{}", ExtcapFormatter(&config)),
///     "arg {number=3}{call=--logfile}{display=Logfile}{tooltip=A file for log messages}{type=fileselect}{mustexist=false}\n"
/// );
/// ```
#[derive(Debug, TypedBuilder)]
pub struct FileSelectConfig {
    pub config_number: u8,
    #[builder(setter(into))]
    pub call: String,
    #[builder(setter(into))]
    pub display: String,
    #[builder(default, setter(strip_option, into))]
    pub tooltip: Option<String>,
    #[builder(default, setter(strip_option, into))]
    pub group: Option<String>,
    #[builder(default = true)]
    pub must_exist: bool,
}

impl ConfigTrait for FileSelectConfig {}
impl<'a> Display for ExtcapFormatter<&'a FileSelectConfig> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "arg {{number={}}}", self.0.config_number)?;
        write!(f, "{{call=--{}}}", self.0.call)?;
        write!(f, "{{display={}}}", self.0.display)?;
        if let Some(tooltip) = &self.0.tooltip {
            write!(f, "{{tooltip={tooltip}}}")?;
        }
        if let Some(group) = &self.0.group {
            write!(f, "{{group={group}}}")?;
        }
        write!(f, "{{type=fileselect}}")?;
        write!(f, "{{mustexist={}}}", self.0.must_exist)?;
        writeln!(f)?;
        Ok(())
    }
}

generate_config_ext!(FileSelectConfig);

/// This provides the possibility to set a true/false value.
///
/// ```
/// use rust_extcap::config::*;
///
/// let config = BooleanConfig::builder()
///     .config_number(2)
///     .call("verify")
///     .display("Verify")
///     .tooltip("Verify package content")
///     .build();
/// assert_eq!(
///     format!("{}", ExtcapFormatter(&config)),
///     "arg {number=2}{call=--verify}{display=Verify}{tooltip=Verify package content}{type=boolean}\n"
/// );
/// ```
#[derive(Debug, TypedBuilder)]
pub struct BooleanConfig {
    pub config_number: u8,
    #[builder(setter(into))]
    pub call: String,
    #[builder(setter(into))]
    pub display: String,
    #[builder(default, setter(strip_option, into))]
    pub tooltip: Option<String>,
    #[builder(default = false)]
    pub default_value: bool,
}

impl ConfigTrait for BooleanConfig {}
impl<'a> Display for ExtcapFormatter<&'a BooleanConfig> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "arg {{number={}}}", self.0.config_number)?;
        write!(f, "{{call=--{}}}", self.0.call)?;
        write!(f, "{{display={}}}", self.0.display)?;
        if let Some(tooltip) = &self.0.tooltip {
            write!(f, "{{tooltip={tooltip}}}")?;
        }
        if self.0.default_value {
            write!(f, "{{default=true}}")?;
        }
        write!(f, "{{type=boolean}}")?;
        writeln!(f)?;
        Ok(())
    }
}

generate_config_ext!(BooleanConfig);

/// This provides the possibility to set a true/false value. boolflag values
/// will only appear in the command line if set to true, otherwise they will not
/// be added to the command-line call for the extcap interface.
///
/// ```
/// use rust_extcap::config::*;
///
/// let config = BoolFlagConfig::builder()
///     .config_number(2)
///     .call("verify")
///     .display("Verify")
///     .tooltip("Verify package content")
///     .build();
/// assert_eq!(
///     format!("{}", ExtcapFormatter(&config)),
///     "arg {number=2}{call=--verify}{display=Verify}{tooltip=Verify package content}{type=boolflag}\n"
/// );
/// ```
// TODO: Combine this with BooleanConfig
#[derive(Debug, TypedBuilder)]
pub struct BoolFlagConfig {
    pub config_number: u8,
    #[builder(setter(into))]
    pub call: String,
    #[builder(setter(into))]
    pub display: String,
    #[builder(default, setter(strip_option, into))]
    pub tooltip: Option<String>,
    #[builder(default = false)]
    pub default_value: bool,
}

impl ConfigTrait for BoolFlagConfig {}
impl<'a> Display for ExtcapFormatter<&'a BoolFlagConfig> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "arg {{number={}}}", self.0.config_number)?;
        write!(f, "{{call=--{}}}", self.0.call)?;
        write!(f, "{{display={}}}", self.0.display)?;
        if let Some(tooltip) = &self.0.tooltip {
            write!(f, "{{tooltip={tooltip}}}")?;
        }
        if self.0.default_value {
            write!(f, "{{default=true}}")?;
        }
        write!(f, "{{type=boolflag}}")?;
        writeln!(f)?;
        Ok(())
    }
}

generate_config_ext!(BoolFlagConfig);

#[derive(Clone, Debug, TypedBuilder)]
pub struct ConfigOptionValue {
    #[builder(setter(into))]
    value: String,
    #[builder(setter(into))]
    display: String,
    #[builder(default = false)]
    default: bool,
}

impl ConfigOptionValue {
    pub fn print_config(&self, number: u8) {
        (self, number).print_config()
    }
}

impl<'a> Display for ExtcapFormatter<&'a (&ConfigOptionValue, u8)> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let (config, arg_number) = self.0;
        write!(f, "value {{arg={}}}", arg_number)?;
        write!(f, "{{value={}}}", config.value)?;
        write!(f, "{{display={}}}", config.display)?;
        write!(f, "{{default={}}}", config.default)?;
        writeln!(f)?;
        Ok(())
    }
}

pub trait ConfigExtGenerated: PrintConfig {
    fn call(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
}

pub trait ConfigTrait: ConfigExtGenerated {}
