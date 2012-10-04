import Text.HTML.TagSoup
import Text.JSON
import Data.List (isInfixOf, isPrefixOf)


-- only extract functions for now

docPath = "/Users/dbp/Code/rust/doc/"

main = do
  let files = map (\n -> ("core::" ++ n, docPath ++ "core/" ++  n ++ ".html")) coreFiles
  parsed <- mapM parseFile files
  writeJson parsed
    where coreFiles = ["option", "num", "os", "path", "run", "str", "int"]

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
