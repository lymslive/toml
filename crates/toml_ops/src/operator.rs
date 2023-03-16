use toml::Value;
use toml::value::Index;
use std::ops::{Div, BitOr, Shl, ShlAssign, Not, Deref, DerefMut};

/// Path segment break on slash(/) or dot(.).
/// eg: `table.subtable.key` or `table/subtable/key` or `array/index/key`
struct PathSegment
{
    paths: Vec<String>,
}

impl PathSegment
{
    /// Resolve path readonly for readonly `toml::Value`.
    fn apply<'tr>(&self, v: &'tr Value) -> Option<&'tr Value> {
        let mut target = Some(v);
        for p in &self.paths {
            if target.is_none() {
                return None;
            }
            if p.is_empty() {
                continue;
            }
            match target.unwrap() {
                Value::Table(table) => { target = table.get(p); },
                Value::Array(array) => {
                    if let Ok(index) = p.parse::<usize>() {
                        target = array.get(index); 
                    }
                },
                _ => { return None; }
            }
        }
        return target;
    }

    /// Resolve path readonly for mutable `toml::Value`.
    /// Bug: if some table key is all numerical char, would mistake as array index.
    fn apply_mut<'tr>(&self, v: &'tr mut Value) -> Option<&'tr mut Value> {
        let mut target = Some(v);
        for p in &self.paths {
            if target.is_none() {
                return None;
            }
            if p.is_empty() {
                continue;
            }
            match p.parse::<usize>() {
                Ok(index) => { target = target.unwrap().get_mut(index); },
                Err(_) => { target = target.unwrap().get_mut(p); },
            }
        }
        return target;
    }
}

/// Type trait that can build `PathSegment` from.
trait PathBuilder {
    fn build_path(&self) -> PathSegment {
        PathSegment { paths: Vec::new() }
    }
}

/// split string to get path segment vector.
impl PathBuilder for &str {
    fn build_path(&self) -> PathSegment {
        let paths = self
            .split(|c| c == '/' || c == '.')
            .map(|s| s.to_string())
            .collect();
        PathSegment { paths }
    }
}

/// usize index only act path on it's own, but cannot split to more path segment.
impl PathBuilder for usize {}

/// Provide toml pointer to supported operator overload.
pub trait PathOperator
{
    /// Construct immutable toml pointer to some initial node.
    fn path<'tr>(&'tr self) -> TomlPtr<'tr>;

    /// Construct immutable toml pointer and move it follwoing sub path.
    fn pathto<'tr>(&'tr self, p: &str) -> TomlPtr<'tr>;

    /// Construct mutable toml pointer to some initial node.
    fn path_mut<'tr>(&'tr mut self) -> TomlPtrMut<'tr>;

    /// Construct mutable toml pointer and move it follwoing sub path.
    fn pathto_mut<'tr>(&'tr mut self, p: &str) -> TomlPtrMut<'tr>;
}

/// Create toml pointer directely from `toml::Value`.
impl PathOperator for Value
{
    fn path<'tr>(&'tr self) -> TomlPtr<'tr> {
        TomlPtr::from(self)
    }
    fn pathto<'tr>(&'tr self, p: &str) -> TomlPtr<'tr> {
        let valop = p.build_path().apply(self);
        TomlPtr { valop }
    }

    fn path_mut<'tr>(&'tr mut self) -> TomlPtrMut<'tr> {
        TomlPtrMut::from(self)
    }
    fn pathto_mut<'tr>(&'tr mut self, p: &str) -> TomlPtrMut<'tr> {
        let valop = p.build_path().apply_mut(self);
        TomlPtrMut { valop }
    }
}

/// Wrapper pointer to `toml::Value` for operator overload.
/// Must refer to an existed toml tree, `Option::None` to refer non-exist node.
#[derive(Copy, Clone)]
pub struct TomlPtr<'tr> {
    valop: Option<&'tr Value>,
}

impl<'tr> TomlPtr<'tr> {
    /// Common constructor
    pub fn new(valop: Option<&'tr Value>) -> Self {
        Self { valop }
    }

    /// As constructor, to build path operand object from a `toml::Value` node.
    pub fn from(v: &'tr Value) -> Self {
        Self::new(Some(v))
    }

    /// Construct new null pointer.
    fn none() -> Self {
        Self::new(None)
    }

    /// Resolve path to sub node.
    fn path<B>(&self, p: B) -> Self where B: PathBuilder + Index + Copy {
        if self.is_none() {
            return Self::none();
        }

        let v = self.unwrap();
        let mut target = v.get(p);
        if target.is_none() {
            let path_segment = p.build_path();
            if path_segment.paths.len() > 1 {
                target = path_segment.apply(v);
            }
        }

        return Self::new(target);
    }
}

/// Overload `!` operator to test the pointer is invalid.
impl<'tr> Not for TomlPtr<'tr> {
    type Output = bool;
    fn not(self) -> Self::Output {
        self.is_none()
    }
}

/// Overload `*` deref operator to treate pointer as `Option<&toml::Value>`.
impl<'tr> Deref for TomlPtr<'tr>
{
    type Target = Option<&'tr Value>;
    fn deref(&self) -> &Self::Target {
        &self.valop
    }
}

/// Path operator `/`, visit sub-node by string key for table or index for array.
/// Can chained as `tomlptr / "path" / "to" / "node"` or `tomlptr / "path/to/node"`.
impl<'tr, Rhs> Div<Rhs> for TomlPtr<'tr>
where Rhs: PathBuilder + Index + Copy
{
    type Output = Self;
    fn div(self, rhs: Rhs) -> Self::Output {
        self.path(rhs)
    }
}

/// Pipe operator `|` with `String`, to get value from string node, 
/// or return `rhs` as default value if pointer is invalid or type mistach.
/// Note that the `rhs` string would be moved.
impl<'tr> BitOr<String> for TomlPtr<'tr>
{
    type Output = String;
    fn bitor(self, rhs: String) -> Self::Output {
        if self.valop.is_none() {
            return rhs;
        }
        match self.valop.unwrap().as_str() {
            Some(s) => s.to_string(),
            None => rhs
        }
    }
}

/// Pipe operator `|` with string literal, to get string value or `rhs` as default.
impl<'tr> BitOr<&'static str> for TomlPtr<'tr>
{
    type Output = &'tr str;
    fn bitor(self, rhs: &'static str) -> Self::Output {
        match self.valop {
            Some(v) => v.as_str().unwrap_or(rhs),
            None => rhs,
        }
    }
}

/// Pipe operator to get integer value or `rhs` as default.
impl<'tr> BitOr<i64> for TomlPtr<'tr>
{
    type Output = i64;
    fn bitor(self, rhs: i64) -> Self::Output {
        match self.valop {
            Some(v) => v.as_integer().unwrap_or(rhs),
            None => rhs,
        }
    }
}

/// Pipe operator to get float value or `rhs` as default.
impl<'tr> BitOr<f64> for TomlPtr<'tr>
{
    type Output = f64;
    fn bitor(self, rhs: f64) -> Self::Output {
        match self.valop {
            Some(v) => v.as_float().unwrap_or(rhs),
            None => rhs,
        }
    }
}

/// Pipe operator to get bool value or `rhs` as default.
impl<'tr> BitOr<bool> for TomlPtr<'tr>
{
    type Output = bool;
    fn bitor(self, rhs: bool) -> Self::Output {
        match self.valop {
            Some(v) => v.as_bool().unwrap_or(rhs),
            None => rhs,
        }
    }
}

/// Mutable version of pointer wrapper of `toml::Value` for operator overload.
/// Must refer to existed toml tree, `Option::None` to refer non-exist node.
/// Note that mutable reference don't support copy.
pub struct TomlPtrMut<'tr> {
    valop: Option<&'tr mut Value>,
}

impl<'tr> TomlPtrMut<'tr> {
    /// Common constructor
    pub fn new(valop: Option<&'tr mut Value>) -> Self {
        Self { valop }
    }

    /// As constructor, to build path operand object from a `toml::Value` node.
    pub fn from(v: &'tr mut Value) -> Self {
        Self::new(Some(v))
    }

    /// Construct new null pointer.
    fn none() -> Self {
        Self::new(None)
    }

    /// Cast to immutable toml pointer.
    fn immut(&mut self) -> TomlPtr<'tr> {
        match self.take() {
            Some(v) => TomlPtr::from(v),
            None => TomlPtr::none(),
        }
    }

    /// Resolve path to sub node.
    fn path<B>(&mut self, p: B) -> Self where B: PathBuilder + Index + Copy {
        if self.is_none() {
            return Self::none();
        }

        let v = self.take().unwrap();

        // Note: use immutable version of get() to determiner path is valid first,
        // otherwise get_mut() and aplly_mut() would trow E0499 as mut ref twice.
        let mut target = None;
        if let Some(_) = v.get(p) {
            target = v.get_mut(p);
        }
        else {
            let path_segment = p.build_path();
            if path_segment.paths.len() > 1 {
                target = path_segment.apply_mut(v);
            }
        }

        return Self::new(target);
    }

    /// Assign any supported value to toml.
    /// But canno overload operator=, will choose <<= instead.
    pub fn assign<T>(&mut self, rhs: T) where Value: From<T> {
        if let Some(ref mut v) = self.valop {
            **v = Value::from(rhs);
        }
    }

    /// Put a value to toml and return pointer to it.
    fn put_val<T>(v: &'tr mut Value, rhs: T) -> Self where Value: From<T> {
        *v = Value::from(rhs);
        Self::from(v)
    }

    /// Put value to string toml node pointer, would invalidate it when type mismatch.
    /// Implement for << String and << &str.
    fn put_string(&mut self, rhs: String) -> Self {
        match self.take() {
            Some(v) if v.is_str() => Self::put_val(v, rhs),
            _ => Self::none()
        }
    }

    /// Implement for << i64.
    fn put_integer(&mut self, rhs: i64) -> Self {
        match self.take() {
            Some(v) if v.is_integer() => Self::put_val(v, rhs),
            _ => Self::none()
        }
    }

    /// Implement for << f64.
    fn put_float(&mut self, rhs: f64) -> Self {
        match self.take() {
            Some(v) if v.is_float() => Self::put_val(v, rhs),
            _ => Self::none()
        }
    }

    /// Implement for << bool.
    fn put_bool(&mut self, rhs: bool) -> Self {
        match self.take() {
            Some(v) if v.is_bool() => Self::put_val(v, rhs),
            _ => Self::none()
        }
    }

    /// Implment for table << (key, val) pair.
    fn push_table<K: ToString, T>(&mut self, key: K, val: T) -> Self where Value: From<T> {
        match self.take() {
            Some(v) if v.is_table() => {
                v.as_table_mut().unwrap().insert(key.to_string(), Value::from(val));
                Self::from(v)
            }
            _ => Self::none()
        }
    }

    /// Implment for array << (val, ) << [item] .
    fn push_array<T>(&mut self, val: T) -> Self where Value: From<T> {
        match self.take() {
            Some(v) if v.is_array() => {
                v.as_array_mut().unwrap().push(Value::from(val));
                Self::from(v)
            }
            _ => Self::none()
        }
    }
}

/// Overload `!` operator to test the pointer is invalid.
impl<'tr> Not for TomlPtrMut<'tr> {
    type Output = bool;
    fn not(self) -> Self::Output {
        self.is_none()
    }
}

/// Overload `*` deref operator to treate pointer as `Option<&mut toml::Value>`.
impl<'tr> Deref for TomlPtrMut<'tr> {
    type Target = Option<&'tr mut Value>;
    fn deref(&self) -> &Self::Target {
        &self.valop
    }
}

/// Overload `*` deref operator to treate pointer as `Option<&mut toml::Value>`.
impl<'tr> DerefMut for TomlPtrMut<'tr> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.valop
    }
}

/// Path operator `/`, visit sub-node by string key for table or index for array.
/// Can chained as `tomlptr / "path" / "to" / "node"` or `tomlptr / "path/to/node"`.
/// Hope to change the node it point to.
impl<'tr, Rhs> Div<Rhs> for TomlPtrMut<'tr>
where Rhs: PathBuilder + Index + Copy
{
    type Output = Self;

    fn div(mut self, rhs: Rhs) -> Self::Output {
        self.path(rhs)
            //TomlPtrMut { valop: path_mut(self.valop, rhs) }
    }
}

/// Pipe operator `|` with `String`, to get value from string node, 
/// or return `rhs` as default value if pointer is invalid or type mistach.
/// Note that the `rhs` string , as well as the pointer itself would be moved.
impl<'tr> BitOr<String> for TomlPtrMut<'tr>
{
    type Output = String;
    fn bitor(mut self, rhs: String) -> Self::Output {
        self.immut().bitor(rhs)
    }
}

/// Pipe operator `|` with string literal, to get string value or `rhs` as default.
impl<'tr> BitOr<&'static str> for TomlPtrMut<'tr> {
    type Output = &'tr str;
    fn bitor(mut self, rhs: &'static str) -> Self::Output {
        self.immut().bitor(rhs)
    }
}

/// Pipe operator to get integer value or `rhs` as default.
impl<'tr> BitOr<i64> for TomlPtrMut<'tr> {
    type Output = i64;
    fn bitor(mut self, rhs: i64) -> Self::Output {
        self.immut().bitor(rhs)
    }
}

/// Pipe operator to get float value or `rhs` as default.
impl<'tr> BitOr<f64> for TomlPtrMut<'tr> {
    type Output = f64;
    fn bitor(mut self, rhs: f64) -> Self::Output {
        self.immut().bitor(rhs)
    }
}

/// Pipe operator to get bool value or `rhs` as default.
impl<'tr> BitOr<bool> for TomlPtrMut<'tr> {
    type Output = bool;
    fn bitor(mut self, rhs: bool) -> Self::Output {
        self.immut().bitor(rhs)
    }
}

/// Operator `<<` to put a string into toml leaf node.
/// While the data type mismatch the node, set self pointer to `None`.
impl<'tr> Shl<&str> for TomlPtrMut<'tr> {
    type Output = Self;
    fn shl(mut self, rhs: &str) -> Self::Output {
        self.put_string(rhs.to_string())
    }
}

/// Operator `<<` to put and move a string into toml leaf node.
/// While the data type mismatch the node, set self pointer to `None`.
impl<'tr> Shl<String> for TomlPtrMut<'tr> {
    type Output = Self;
    fn shl(mut self, rhs: String) -> Self::Output {
        self.put_string(rhs)
    }
}

/// Operator `<<` to put a integer value into toml leaf node.
/// While the data type mismatch the node, set self pointer to `None`.
impl<'tr> Shl<i64> for TomlPtrMut<'tr> {
    type Output = Self;
    fn shl(mut self, rhs: i64) -> Self::Output {
        self.put_integer(rhs)
    }
}

/// Operator `<<` to put a float value into toml leaf node.
/// While the data type mismatch the node, set self pointer to `None`.
impl<'tr> Shl<f64> for TomlPtrMut<'tr> {
    type Output = Self;
    fn shl(mut self, rhs: f64) -> Self::Output {
        self.put_float(rhs)
    }
}

/// Operator `<<` to put a bool value into toml leaf node.
/// While the data type mismatch the node, set self pointer to `None`.
impl<'tr> Shl<bool> for TomlPtrMut<'tr> {
    type Output = Self;
    fn shl(mut self, rhs: bool) -> Self::Output {
        self.put_bool(rhs)
    }
}

/// Operator `<<` to push key-value pair (tuple) into toml table.
/// eg: `toml/table/node << (k, v)` where the k v will be moved.
impl<'tr, K: ToString, T> Shl<(K, T)> for TomlPtrMut<'tr> where Value: From<T> {
    type Output = Self;
    fn shl(mut self, rhs: (K, T)) -> Self::Output {
        self.push_table(rhs.0, rhs.1)
    }
}

/// Operator `<<` to push one value tuple into toml array.
/// eg: `toml/array/node << (v,)`.
/// Note that use single tuple to distinguish with pushing scalar to leaf node.
impl<'tr, T> Shl<(T,)> for TomlPtrMut<'tr> where Value: From<T> {
    type Output = Self;
    fn shl(mut self, rhs: (T,)) -> Self::Output {
        self.push_array(rhs.0)
    }
}

/// Operator `<<` to push one item to toml array.
/// eg: `toml/array/node << [v1]`
impl<'tr, T: Copy> Shl<[T;1]> for TomlPtrMut<'tr> where Value: From<T> {
    type Output = Self;
    fn shl(mut self, rhs: [T;1]) -> Self::Output {
        self.push_array(rhs[0])
    }
}

/// Operator `<<` to push a slice to toml array.
/// eg: `toml/array/node << &[v1, v2, v3, ...][..]`
impl<'tr, T: Copy> Shl<&[T]> for TomlPtrMut<'tr> where Value: From<T> {
    type Output = Self;
    fn shl(mut self, rhs: &[T]) -> Self::Output {
        for item in rhs {
            self = self.push_array(*item);
        }
        self
    }
}

/// Operator `<<=` re-assign to an node unconditionally, may change it data type.
/// Note donot use chained `<<=` as `<<` can because `<<=` is right associated.
impl<'tr, T> ShlAssign<T> for TomlPtrMut<'tr> where Value: From<T> {
    fn shl_assign(&mut self, rhs: T) {
        self.assign(rhs);
    }
}

#[cfg(test)]
mod tests; // { move to tests.rs }
