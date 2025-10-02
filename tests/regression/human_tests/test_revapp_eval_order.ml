let f = fun x -> x * 10;
print (print 0; f) (print 1; 2);
print (print 3; 99) |> (print 4; f);