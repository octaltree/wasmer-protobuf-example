function bench()
  local x = 0
  for i=1,300000 do
    x = tostring(i):len()
  end
end

bench()
