#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    Star(usize),
    Var(String, usize),
    Arrow(Box<Type>, Box<Type>),
    ForAll(String, Box<Type>, Box<Type>),
    Lambda(String, Box<Type>, Box<Type>),
    App(Box<Type>, Box<Type>),
    Remote(String),
}

impl Type {
    pub fn shift(&self, name: &str, p: usize) -> Type {
        match self {
            Type::Var(n, i) if n == name && *i >= p => Type::Var(n.clone(), i + 1),
            Type::ForAll(n, i, o) => Type::ForAll(
                n.clone(),
                Box::new(i.shift(name, p)),
                Box::new(o.shift(name, p + 1)),
            ),
            Type::Lambda(n, i, o) => Type::Lambda(
                n.clone(),
                Box::new(i.shift(name, p)),
                Box::new(o.shift(name, p + 1)),
            ),
            Type::Arrow(l, r) => Type::Arrow(Box::new(l.shift(name, p)), Box::new(r.shift(name, p))),
            Type::App(f, a) => Type::App(Box::new(f.shift(name, p)), Box::new(a.shift(name, p))),
            _ => self.clone(),
        }
    }

    pub fn subst(&self, name: &str, value: &Type, l: usize) -> Type {
        match self {
            Type::Arrow(i, o) => Type::Arrow(
                Box::new(i.subst(name, value, l)),
                Box::new(o.subst(name, value, l)),
            ),
            Type::ForAll(n, i, o) => Type::ForAll(
                n.clone(),
                Box::new(i.subst(name, value, l)),
                Box::new(o.subst(name, &value.shift(n, 0), l + 1)),
            ),
            Type::Lambda(n, i, o) => Type::Lambda(
                n.clone(),
                Box::new(i.subst(name, value, l)),
                Box::new(o.subst(name, &value.shift(n, 0), l + 1)),
            ),
            Type::App(f, a) => Type::App(
                Box::new(f.subst(name, value, l)),
                Box::new(a.subst(name, value, l)),
            ),
            Type::Var(n, i) if n == name && *i == l => value.clone(),
            Type::Var(n, i) if n == name && *i > l => Type::Var(n.clone(), i - 1),
            _ => self.clone(),
        }
    }

    pub fn norm(&self) -> Type {
        match self {
            Type::Arrow(i, o) => Type::Arrow(Box::new(i.norm()), Box::new(o.norm())),
            Type::ForAll(n, i, o) => Type::ForAll(
                n.clone(),
                Box::new(i.norm()),
                Box::new(o.norm()),
            ),
            Type::Lambda(n, i, o) => Type::Lambda(
                n.clone(),
                Box::new(i.norm()),
                Box::new(o.norm()),
            ),
            Type::App(f, a) => match f.norm() {
                Type::Lambda(n, _, o) => o.subst(&n, &a.norm(), 0).norm(),
                nf => Type::App(Box::new(nf), Box::new(a.norm())),
            },
            _ => self.clone(),
        }
    }

    pub fn eq(&self, other: &Type) -> bool {
        match (self, other) {
            (Type::Arrow(i1, o1), Type::Arrow(i2, o2)) => i1.eq(i2) && o1.eq(o2),
            (Type::ForAll(n1, i1, o1), Type::ForAll(n2, i2, o2)) => {
                i1.eq(i2) && o1.eq(&Box::new(o2.subst(n2, &Type::Var(n1.clone(), 0), 0)))
            }
            (Type::Lambda(n1, i1, o1), Type::Lambda(n2, i2, o2)) => {
                i1.eq(i2) && o1.eq(&Box::new(o2.subst(n2, &Type::Var(n1.clone(), 0), 0)))
            }
            (Type::App(f1, a1), Type::App(f2, a2)) => f1.eq(f2) && a1.eq(a2),
            (Type::Star(n1), Type::Star(n2)) => n1 == n2,
            (Type::Var(n1, i1), Type::Var(n2, i2)) => n1 == n2 && i1 == i2,
            (Type::Remote(n1), Type::Remote(n2)) => n1 == n2,
            _ => false,
        }
    }

    pub fn type_check(&self, context: &[(String, Type)]) -> Result<Type, String> {
        match self {
            Type::Star(n) => Ok(Type::Star(n + 1)),
            Type::Var(n, _) => {
                context
                    .iter()
                    .find(|(name, _)| name == n)
                    .ok_or_else(|| format!("free variable: {}", n))
                    .map(|(_, t)| t.clone())
            }
            Type::Arrow(i, o) => {
                let i_type = i.type_check(context)?;
                let o_type = o.type_check(context)?;
                Ok(Type::Star(std::cmp::max(i_type.star_level()?, o_type.star_level()?)))
            }
            Type::ForAll(n, i, o) => {
                let i_type = i.type_check(context)?;
                let mut new_context = context.to_vec();
                new_context.push((n.clone(), i_type.norm()));
                let o_type = o.type_check(&new_context)?;
                Ok(Type::Star(std::cmp::max(i_type.star_level()?, o_type.star_level()?)))
            }
            Type::Lambda(n, i, o) => {
                let i_type = i.type_check(context)?;
                let mut new_context = context.to_vec();
                new_context.push((n.clone(), i_type.norm()));
                let o_type = o.type_check(&new_context)?;
                Ok(Type::Arrow(Box::new(i_type), Box::new(o_type)))
            }
            Type::App(f, a) => {
                let f_type = f.type_check(context)?;
                if let Type::ForAll(n, i, o) = f_type {
                    let a_type = a.type_check(context)?;
                    if i.eq(&Box::new(a_type.clone())) {
                        Ok(o.subst(&n, &a_type, 0).norm())
                    } else {
                        Err(format!("Type mismatch: expected {:?}, found {:?}", i, a_type.clone()))
                    }
                } else {
                    Err(format!("Expected a function, found {:?}", f_type))
                }
            }
            Type::Remote(n) => Err(format!("Remote type checking not implemented for {:?}", n)),
        }
    }

    fn star_level(&self) -> Result<usize, String> {
        match self {
            Type::Star(n) => Ok(*n),
            _ => Err(format!("Not a star type: {:?}", self)),
        }
    }
}