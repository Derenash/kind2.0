use crate::{*};

use std::collections::BTreeMap;
use std::collections::BTreeSet;

impl Book {

  pub fn to_hvm1(&self) -> String {
    todo!()
    //let mut used = BTreeSet::new();
    //let mut code = String::new();
    //for (name, term) in &self.defs {
      //let metas = term.count_metas();
      //let mut lams = String::new();
      //for i in 0 .. term.count_metas() {
        //lams = format!("{} λ_{}", lams, i);
      //}
      //let subs = (0 .. metas).map(|h| format!("(Pair \"{}\" None)", h)).collect::<Vec<_>>().join(",");
      //code.push_str(&format!("Book.{} = (Ref \"{}\" [{}] {}{})\n", name, name, subs, lams, term.to_hvm1(im::Vector::new(), &mut 0)));
      //used.insert(name.clone());
    //}
    //code
  }

  pub fn to_hs(&self) -> String {
    let mut code = String::new();
    for (name, term) in &self.defs {
      let expr = term.to_hs(im::Vector::new(), &mut 0);
      code.push_str(&format!("{} = (Ref \"{}\" {})\n", Term::to_hs_name(name), name, expr));
    }
    code
  }

}
