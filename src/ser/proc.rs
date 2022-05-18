use crate::{
    ir::{
        expr::Expr,
        proc::{Procedure, Statement},
    },
    ser::ProgramCtx,
    uid::Uid,
};
use serde_json::{json, Value as Json};
use std::{cell::RefCell, collections::HashMap};

pub(super) fn serialize_procs(
    ctx: &ProgramCtx,
    procs: &HashMap<String, Procedure>,
) -> HashMap<Uid, Json> {
    let ctx = ProcCtx {
        inner: ctx,
        blocks: Default::default(),
    };
    for (name, proc) in procs {
        ctx.serialize_proc(name, proc);
    }
    ctx.blocks.into_inner()
}

struct ProcCtx<'a> {
    inner: &'a ProgramCtx,
    blocks: RefCell<HashMap<Uid, Json>>,
}

impl<'a> ProcCtx<'a> {
    fn serialize_proc(&self, name: &str, proc: &Procedure) {
        match name {
            "when-flag-clicked" => {
                assert!(proc.params.is_empty());
                let this = self.new_uid();
                let (body, _) = self.serialize_stmt(&proc.body, this, None);
                self.emit_block(
                    this,
                    json!({
                        "opcode": "event_whenflagclicked",
                        "next": body,
                        "parent": null,
                        "topLevel": true,
                        "x": 0,
                        "y": 0,
                    }),
                );
            }
            "when-cloned" => todo!(),
            "when-received" => todo!(),
            _ => todo!(),
        }
    }

    fn serialize_stmt(
        &self,
        stmt: &Statement,
        parent: Uid,
        next: Option<Uid>,
    ) -> (Option<Uid>, Option<Uid>) {
        match stmt {
            Statement::ProcCall { proc_name, args } => todo!(),
            Statement::Do(stmts) => match &stmts[..] {
                [] => (None, Some(parent)),
                [single] => self.serialize_stmt(single, parent, next),
                _ => {
                    todo!() // How can we do this without ugly hacks?
                }
            },
            Statement::IfElse {
                condition,
                if_true,
                if_false,
            } => todo!(),
            Statement::Repeat { times, body } => self.emit_stacking(
                "control_repeat",
                parent,
                next,
                &[
                    ("TIMES", &self.expr_input(times)),
                    ("SUBSTACK", &self.stmt_input(body)),
                ],
                &[],
            ),
            Statement::Forever(body) => {
                assert!(next.is_none());
                self.emit_stacking(
                    "control_forever",
                    parent,
                    next,
                    &[("SUBSTACK", &self.stmt_input(body))],
                    &[],
                )
            }
            Statement::Until { condition, body } => self.emit_stacking(
                "control_repeat_until",
                parent,
                next,
                &[
                    ("CONDITION", &self.expr_input(condition)),
                    ("SUBSTACK", &self.stmt_input(body)),
                ],
                &[],
            ),
            Statement::While { condition, body } => self.emit_stacking(
                "control_while",
                parent,
                next,
                &[
                    ("CONDITION", &self.expr_input(condition)),
                    ("SUBSTACK", &self.stmt_input(body)),
                ],
                &[],
            ),
            Statement::For {
                counter,
                times,
                body,
            } => todo!(),
        }
    }

    fn serialize_expr(&self, expr: &Expr, parent: Uid) -> Json {
        match expr {
            Expr::Lit(lit) => json!([10, lit.to_cow_str()]),
            Expr::Sym(_) => todo!(),
            Expr::FuncCall(_, _) => todo!(),
        }
    }

    fn stmt_input<'s>(
        &'s self,
        stmt: &'s Statement,
    ) -> impl Fn(Uid) -> Json + 's {
        |this| json!(self.serialize_stmt(stmt, this, None).0)
    }

    fn expr_input<'s>(&'s self, expr: &'s Expr) -> impl Fn(Uid) -> Json + 's {
        |this| self.serialize_expr(expr, this)
    }

    fn emit_stacking(
        &self,
        opcode: &str,
        parent: Uid,
        next: Option<Uid>,
        inputs: &[(&str, &dyn Fn(Uid) -> Json)],
        fields: &[(&str, &dyn Fn(Uid) -> Json)],
    ) -> (Option<Uid>, Option<Uid>) {
        let this = self.new_uid();

        let inputs = Json::Object(
            inputs
                .iter()
                .copied()
                .map(|(name, fun)| (name.to_owned(), fun(this)))
                .collect(),
        );
        let fields = Json::Object(
            fields
                .iter()
                .copied()
                .map(|(name, fun)| (name.to_owned(), fun(this)))
                .collect(),
        );

        self.emit_block(
            this,
            json!({
                "opcode": opcode,
                "next": next,
                "parent": parent,
                "inputs": inputs,
                "fields": fields,
            }),
        );

        (Some(this), Some(this))
    }

    fn emit_block(&self, uid: Uid, block: Json) {
        self.blocks.borrow_mut().insert(uid, block);
    }

    fn new_uid(&self) -> Uid {
        self.inner.new_uid()
    }
}
