use crate::project::CommitPlan;

pub mod deps;
pub mod labels;
pub mod pr;
pub mod skill;
pub mod todos;

pub fn report(plan: &CommitPlan) {
    if plan.commit {
        return;
    }
    if plan.others.is_empty() {
        println!("not committed (--no-commit) - the changes are in the working tree");
        return;
    }
    crate::logger::warn(&format!(
        "not committed - the checkout has other changes: {}. The changes of todoer are in the working tree. Commit them yourself",
        plan.others.join(", ")
    ));
}
