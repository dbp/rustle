import Text.HTML.TagSoup
import Text.JSON
import Data.List (isInfixOf, isPrefixOf)


-- only extract functions for now

docPath = "/Users/dbp/Code/rust/doc/"

main = do
  let coreFiles = map (\n -> ("core::" ++ n, docPath ++ "core/" ++  n ++ ".html")) coreFileList
  --let stdFiles = map (\n -> ("std::" ++ n, docPath ++ "std/" ++  n ++ ".html")) stdFileList
  parsedCore <- mapM parseFile coreFiles
  --parsedStd <- mapM parseFile stdFiles
  writeJson (parsedCore) -- ++ parsedStd)
    -- produced with the hack:
    -- cat index.html | grep Module | awk -F '<code>' '{print $2}' | awk -F '<' '{print $1}'
    where coreFileList = ["at_vec", "bool", "box", "cast", "char", "cmp", "comm", "dlist", "dlist_iter", "dvec", "dvec_iter", "either", "f32", "f64", "flate", "float", "from_str", "future", "gc", "hash", "i16", "i32", "i64", "i8", "int", "io", "iter", "libc", "logging", "mutable", "num", "option", "option_iter", "os", "path", "pipes", "ptr", "rand", "reflect", "repr", "result", "run", "send_map", "str", "sys", "task", "to_bytes", "to_str", "tuple", "u16", "u32", "u64", "u8", "uint", "uniq", "unit", "util", "vec"]
          stdFileList = ["arc","arena","base64","bitv","c_vec","cell","cmp","comm","dbg","deque","ebml","ebml2","fun_treemap","getopts","json","list","map","md4","net","net_ip","net_tcp","net_url","par","prettyprint","prettyprint2","rope","serialization","serialization2","sha1","smallintmap","sort","sync","tempfile","term","time","timer","treemap","uv","uv_global_loop","uv_iotask","uv_ll"]
writeJson files = do
  let dat = encode $ JSArray $ concat $ map writeJson' files
  writeFile "rustle.data" dat
    where writeJson' = map (\(a,n,t,d,p) -> JSObject $
                                            toJSObject [("anchor", JSString $ toJSString a)
                                                       ,("name",   JSString $ toJSString n)
                                                       ,("type",   JSString $ toJSString t)
                                                       ,("desc",   JSString $ toJSString d)
                                                       ,("path",   JSString $ toJSString p)])

parseFile (path, n) = do
    f <- readFile n
    let tags = parseTags f
    let sections = partitions (\t -> (isTagOpenName "div" t) &&
                                    ("section" `isInfixOf` (fromAttrib "class" t)) &&
                                    ("function" `isPrefixOf` (fromAttrib "id" t)))
                              tags
    return $ map (extractInfo path) sections



-- div's id gives you link
-- h2 > code has name
-- pre > code has type
-- first p has short description
extractInfo path tags = (anchor, name, ty, desc, path)
    where anchor   = fromAttrib "id" (head tags)
          name     = getCod $ getTag "h2" tags
          ty       = getCod $ getTag "pre" tags
          desc     = getText $ headSafe $ drop 1 $ getTag "p" tags
          getCod   = getText . headSafe . drop 1 . (getTag "code")
          getTag n = dropWhile (not . isTagOpenName n)
          getText = maybe "" fromTagText
          headSafe [] = Nothing
          headSafe x = Just (head x)
