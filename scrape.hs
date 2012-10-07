import Text.HTML.TagSoup
import Text.JSON
import Data.List (isInfixOf, isPrefixOf)
import System.Environment (getArgs)

-- This is a really simple scraper that is intended to be replaced as soon as
-- rustdoc is a library and we can use it directly, instead of using it to generate
-- html (which we don't want) and then scraping the html. it is written in haskell
-- because as of now, AFAIK, there is no html scraper written in rust, and I wanted
-- to work on rustle, not an html scraper :)


main = do
  docPath <- fmap head getArgs
  let coreFiles = map (\n -> ("core::" ++ n, docPath ++ "/core/" ++  n ++ ".html")) coreFileList
  parsedCore <- mapM parseFile coreFiles
  writeJson (parsedCore) -- ++ parsedStd)
    -- produced with the hack:
    -- cat index.html | grep Module | awk -F '<code>' '{print $2}' | awk -F '<' '{print $1}'
    -- note that the order they appear is the order results will appear in, equally matching
    -- queries, so the order here is intentional (and subjective).
    where coreFileList = ["str", "vec", "option", "bool", "io", "os", "path", "either", "run", "at_vec", "box", "cast", "char", "cmp", "comm", "dlist", "dlist_iter", "dvec", "dvec_iter",  "f32", "f64", "flate", "float", "from_str", "future", "gc", "hash", "i16", "i32", "i64", "i8", "int", "iter", "libc", "logging", "mutable", "num", "option_iter", "pipes", "ptr", "rand", "reflect", "repr", "result",  "send_map", "sys", "task", "to_bytes", "to_str", "tuple", "u16", "u32", "u64", "u8", "uint", "uniq", "unit", "util"]
writeJson files = do
  let dat = encode $ JSArray $ concat $ map writeJson' files
  writeFile "rustle.data" dat
    where writeJson' = map (\(a,n,t,s,d,p) -> JSObject $
                                            toJSObject [("anchor", JSString $ toJSString a)
                                                       ,("name",   JSString $ toJSString n)
                                                       ,("type",   JSString $ toJSString t)
                                                       ,("self",   JSString $ toJSString s)
                                                       ,("desc",   JSString $ toJSString d)
                                                       ,("path",   JSString $ toJSString p)])

parseFile (path, n) = do
    f <- readFile n
    let tags = parseTags f
    let sects = partitions (\t -> (isTagOpenName "div" t) &&
                                      ("level2" `isInfixOf` (fromAttrib "class" t)) &&
                                      (("function" `isPrefixOf` (fromAttrib "id" t)) ||
                                       ("implementation" `isPrefixOf` (fromAttrib "id" t))))
                              tags
    return $ concat $ map (extract path) sects

extract path tags = if ("function" `isPrefixOf` (fromAttrib "id" (head tags)))
                    then extractFunc path tags
                    else extractMethods path tags

-- for now, ignore impls of traits - they add lots of results and with the current
-- presentation, dillute the results
extractMethods path tags = if isExtensions then map (extractMethod path clas self) methods
                                           else []
  where methods = partitions (\t -> (isTagOpenName "div" t) &&
                      ("level3" `isInfixOf` (fromAttrib "class" t)) &&
                      ("method" `isPrefixOf` (fromAttrib "id" t)))
               tags
        impl = partitions (isTagOpenName "code") $
          takeWhile (not.isTagCloseName "h2") $ getTag "h2" tags
        isExtensions = (length impl) == 1
        self = getCod $ impl !! (if isExtensions then 0 else 1)
        clas = if isExtensions then self else getCod $ impl !! 0

extractMethod path clas self tags = (anchor, name, ty, self, desc, path)
  where anchor = fromAttrib "id" (head tags)
        name   = getCod $ getTag "h3" tags
        ty     = getCod $ getTag "pre" tags
        desc   = "a method of " ++ clas ++ ": " ++
          (getText $ headSafe $ drop 1 $ getTag "p" tags)

-- div's id gives you link
-- h2 > code has name
-- pre > code has type
-- first p has short description
extractFunc path tags = [(anchor, name, ty, "", desc, path)]
    where anchor = fromAttrib "id" (head tags)
          name   = getCod $ getTag "h2" tags
          ty     = getCod $ getTag "pre" tags
          desc   = getText $ headSafe $ drop 1 $ getTag "p" tags


getCod   = getText . headSafe . drop 1 . (getTag "code")
getTag n = dropWhile (not . isTagOpenName n)
getText = maybe "" fromTagText
headSafe [] = Nothing
headSafe x = Just (head x)