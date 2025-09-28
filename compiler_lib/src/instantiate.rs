use std::collections::HashMap;

use crate::ast::StringId;
use crate::core::*;
use crate::parse_types::SourceLoc;

pub enum Substitutions<'a> {
    Type(&'a HashMap<StringId, (Value, Use)>),
    Abs(&'a HashMap<StringId, TypeCtorInd>),
}

pub struct InstantionContext<'a> {
    core: &'a mut TypeCheckerCore,
    subs: Substitutions<'a>,

    root: SourceLoc,
    vmap: HashMap<Value, Value>,
    umap: HashMap<Use, Use>,
}
impl InstantionContext<'_> {
    pub fn new<'a>(core: &'a mut TypeCheckerCore, subs: Substitutions<'a>, root: SourceLoc) -> InstantionContext<'a> {
        let mut new = InstantionContext {
            core,
            subs,
            root,
            vmap: HashMap::new(),
            umap: HashMap::new(),
        };
        // Add bot and top to the map so that we never try to deref them
        // since they aren't valid indices.
        new.vmap.insert(new.core.bot(), new.core.bot());
        new.umap.insert(new.core.top_use(), new.core.top_use());
        new
    }

    pub fn instantiate_val(&mut self, old: Value) -> Value {
        if let Some(new) = self.vmap.get(&old) {
            return *new;
        }

        let node = match self.core.r.get(old.0).unwrap() {
            TypeNode::Value(node) => node.clone(),
            TypeNode::Var { .. } => return old,
            _ => unreachable!(),
        };
        // Check if type actually depends on the variables being instantiated, if not return unchanged.
        if !node.2.get(self.root) {
            self.vmap.insert(old, old);
            return old;
        }
        use VTypeHead::*;
        if let VTypeVar(spec) = node.0 {
            // If root doesn't match loc, then it shouldn't be in deps and we'd have returned above
            assert!(spec.loc == self.root);
            let new = match self.subs {
                Substitutions::Type(m) => m.get(&spec.name).unwrap().0,
                Substitutions::Abs(m) => {
                    let ty_cond = *m.get(&spec.name).unwrap();
                    self.core.simple_val(ty_cond, node.1)
                }
            };
            self.vmap.insert(old, new);
            return new;
        }

        let ph = self.core.val_placeholder();
        self.vmap.insert(old, ph);

        let head = match node.0 {
            VInstantiateExist { .. } | VTop | VAbstract { .. } | VTypeVar(..) => unreachable!(),

            VUnion(vals) => VUnion(vals.into_iter().map(|v| self.instantiate_val(v)).collect()),

            VFunc { arg, ret } => VFunc {
                arg: self.instantiate_use(arg),
                ret: self.instantiate_val(ret),
            },

            VObj { fields } => VObj {
                fields: fields
                    .into_iter()
                    .map(|(k, (rty, wty, span))| {
                        (k, (self.instantiate_val(rty), wty.map(|w| self.instantiate_use(w)), span))
                    })
                    .collect(),
            },

            VCase { case: (tag, ty) } => VCase {
                case: (tag, self.instantiate_val(ty)),
            },

            VPolyHead(poly, ty, poison) => {
                let poison = poison || matches!(self.subs, Substitutions::Type(..));
                VPolyHead(poly, self.instantiate_val(ty), poison)
            }

            VDisjointIntersect(vars, default) => {
                assert!(!vars.iter().any(|spec| spec.loc == self.root));
                VDisjointIntersect(vars.clone(), default.clone().map(|t| self.instantiate_val(t)))
            }
        };

        let mut deps = node.2;
        deps.remove(self.root);
        self.core.set_val(ph, head, node.1, Some(deps));
        ph
    }

    pub fn instantiate_use(&mut self, old: Use) -> Use {
        if let Some(new) = self.umap.get(&old) {
            return *new;
        }

        let node = match self.core.r.get(old.0).unwrap() {
            TypeNode::Use(node) => node.clone(),
            TypeNode::Var { .. } => return old,
            _ => unreachable!(),
        };
        // Check if type actually depends on the variables being instantiated, if not return unchanged.
        if !node.2.get(self.root) {
            self.umap.insert(old, old);
            return old;
        }
        use UTypeHead::*;
        if let UTypeVar(spec) = node.0 {
            // If root doesn't match loc, then it shouldn't be in deps and we'd have returned above
            assert!(spec.loc == self.root);
            let new = match self.subs {
                Substitutions::Type(m) => m.get(&spec.name).unwrap().1,
                Substitutions::Abs(m) => {
                    let ty_cond = *m.get(&spec.name).unwrap();
                    self.core.simple_use(ty_cond, node.1)
                }
            };
            self.umap.insert(old, new);
            return new;
        }

        let ph = self.core.use_placeholder();
        self.umap.insert(old, ph);

        let head = match node.0 {
            UInstantiateUni { .. } | UBot | UAbstract { .. } | UTypeVar(..) => unreachable!(),

            UIntersection(uses) => UIntersection(uses.into_iter().map(|u| self.instantiate_use(u)).collect()),

            UFunc { arg, ret } => UFunc {
                arg: self.instantiate_val(arg),
                ret: self.instantiate_use(ret),
            },

            UObj { fields } => UObj {
                fields: fields
                    .into_iter()
                    .map(|(k, (rty, wty, span))| {
                        (k, (self.instantiate_use(rty), wty.map(|w| self.instantiate_val(w)), span))
                    })
                    .collect(),
            },

            UCase { cases, wildcard } => {
                assert!(wildcard.is_none());
                UCase {
                    cases: cases.into_iter().map(|(n, ty)| (n, self.instantiate_use(ty))).collect(),
                    wildcard: None,
                }
            }

            UPolyHead(poly, ty, poison) => {
                let poison = poison || matches!(self.subs, Substitutions::Type(..));
                UPolyHead(poly, self.instantiate_use(ty), poison)
            }

            UDisjointUnion(vars, default) => {
                assert!(!vars.iter().any(|spec| spec.loc == self.root));
                UDisjointUnion(vars.clone(), default.clone().map(|t| self.instantiate_use(t)))
            }
        };

        let mut deps = node.2;
        deps.remove(self.root);
        self.core.set_use(ph, head, node.1, Some(deps));
        ph
    }
}
