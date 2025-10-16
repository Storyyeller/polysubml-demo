let _ = fun x -> (
  let a: rec t=t * (type Q. Q -> Q) = (x.a : rec t=t * (type Q. Q -> Q));
  let b: rec t=((type Q. Q -> Q) * ((type Q. Q -> Q) * t)) = (x.b : rec t=((type Q. Q -> Q) * ((type Q. Q -> Q) * ((type Q. Q -> Q) * t))));

  let c: rec t=((type W. W -> W*W) * ((type W. W -> W*any) * t)) = (x.c : rec t=((type W. W -> W*W) * ((type W. W -> W*W) * ((type W. W -> never*W) * t))));

 x 
)