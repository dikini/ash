-- Ash Core AST Types
-- Defines Value, Expr, Pattern, and related types per SPEC-021 Section 5

import Lean

namespace Ash

open Lean

mutual

/- ## Value Type
Runtime representation of values in Ash.
Matches SPEC-021 Section 5.1 and SPEC-004 Section 2 (Semantic Domains).
-/
inductive Value where
  | int (i : Int)
  | string (s : String)
  | bool (b : Bool)
  | null
  | time (t : String)  -- ISO 8601 timestamp per SPEC-004
  | ref (r : String)   -- Reference/identifier per SPEC-004
  | cap (c : String)   -- Capability reference per SPEC-004
  | list (vs : List Value)
  | record (fields : List (String × Value))
  | variant (variant_name : String) (fields : List (String × Value))
  | tuple (elements : List Value)
  deriving Repr, BEq, Inhabited

/- ## Pattern Type
Patterns for destructuring values.
Matches SPEC-021 Section 5.1.
-/
inductive Pattern where
  | wildcard
  | variable (name : String)
  | literal (v : Value)
  | variant (name : String) (fields : List (String × Pattern))
  | tuple (elements : List Pattern)
  | record (fields : List (String × Pattern))
  deriving Repr, BEq, Inhabited

/- ## MatchArm Structure
A single arm in a match expression.
-/
structure MatchArm where
  pattern : Pattern
  body : Expr
  deriving Repr, BEq

/- ## Expr Type
Expression AST nodes.
Matches SPEC-021 Section 5.1.
-/
inductive Expr where
  | literal (v : Value)
  | variable (name : String)
  | constructor (name : String) (fields : List (String × Expr))
  | tuple (elements : List Expr)
  | match (scrutinee : Expr) (arms : List MatchArm)
  | if_let (pattern : Pattern) (expr : Expr) (then_branch : Expr) (else_branch : Expr)
  deriving Repr, BEq, Inhabited

end

-- Manual Inhabited instance for MatchArm since it references Expr
instance : Inhabited MatchArm where
  default := { pattern := Pattern.wildcard, body := Expr.literal (Value.int 0) }

/- ## Type Definition Types
Type definitions for ADTs.
Matches SPEC-021 Section 5.2.
-/

-- Type expression for type definitions
inductive TypeExpr where
  | named (name : String)
  | var (id : Nat)
  | constructor (name : String) (args : List TypeExpr)
  deriving Repr, BEq, Inhabited

-- Enum variant definition
structure Variant where
  name : String
  fields : List (String × TypeExpr)
  deriving Repr, BEq, Inhabited

-- Type body (enum or struct)
inductive TypeBody where
  | enum (variants : List Variant)
  | struct (fields : List (String × TypeExpr))
  deriving Repr, BEq, Inhabited

-- Type definition
structure TypeDef where
  name : String
  params : List Nat
  body : TypeBody
  deriving Repr, BEq, Inhabited

/- ## JSON Serialization
ToJson and FromJson instances for all AST types.
Matches SPEC-021 Section 7.1.
-/

namespace Value

partial def toJson : Value → Json
  | .int i => Json.mkObj [("type", "int"), ("value", Json.num i)]
  | .string s => Json.mkObj [("type", "string"), ("value", s)]
  | .bool b => Json.mkObj [("type", "bool"), ("value", b)]
  | .null => Json.mkObj [("type", "null")]
  | .time t => Json.mkObj [("type", "time"), ("value", t)]
  | .ref r => Json.mkObj [("type", "ref"), ("value", r)]
  | .cap c => Json.mkObj [("type", "cap"), ("value", c)]
  | .list vs => Json.mkObj [("type", "list"), ("value", Json.arr (vs.map toJson).toArray)]
  | .record fields =>
      -- Rust-compatible: plain JSON object {"x": 1, "y": 2}
      fields.foldl (fun acc (n, v) => acc.setObjVal! n (toJson v)) (Json.mkObj [])
  | .variant vn fields =>
      let obj := fields.foldl (fun acc (n, v) => acc.setObjVal! n (toJson v)) (Json.mkObj [])
      Json.mkObj [("type", "variant"), ("variant_name", vn), ("fields", obj)]
  | .tuple elems =>
      Json.mkObj [("type", "tuple"), ("elements", Json.arr (elems.map toJson).toArray)]

instance : ToJson Value where
  toJson := toJson

mutual

partial def fromJson (json : Json) : Except String Value :=
  -- Try to get type field; if not present, treat as record
  match json.getObjValAs? String "type" with
  | .ok type => fromJsonWithType type json
  | .error _ =>
      -- No type field: plain object is a record
      fromJsonRecord json

partial def fromJsonRecord (json : Json) : Except String Value := do
  let obj ← json.getObj?
  let fields ← obj.toList.mapM (fun (k, v) => do
    let val ← fromJson v
    pure (k, val))
  pure (.record fields)

partial def fromJsonWithType (type : String) (json : Json) : Except String Value := do
  match type with
  | "int" =>
      let v ← json.getObjValAs? Int "value"
      pure (.int v)
  | "string" =>
      let v ← json.getObjValAs? String "value"
      pure (.string v)
  | "bool" =>
      let v ← json.getObjValAs? Bool "value"
      pure (.bool v)
  | "null" =>
      pure .null
  | "time" =>
      let v ← json.getObjValAs? String "value"
      pure (.time v)
  | "ref" =>
      let v ← json.getObjValAs? String "value"
      pure (.ref v)
  | "cap" =>
      let v ← json.getObjValAs? String "value"
      pure (.cap v)
  | "list" =>
      let arr ← json.getObjVal? "value"
      let elems ← arr.getArr?
      let vs ← elems.mapM fromJson
      pure (.list vs.toList)
  | "record" => throw "Record JSON format is a plain object, use object parser directly"
  | "variant" =>
      let vn ← json.getObjValAs? String "variant_name"
      let fieldsJson ← json.getObjVal? "fields"
      let fields ← parseFields fieldsJson
      pure (.variant vn fields)
  | "tuple" =>
      let arr ← json.getObjVal? "elements"
      let elems ← arr.getArr?
      let vs ← elems.mapM fromJson
      pure (.tuple vs.toList)
  | _ => throw s!"Unknown value type: {type}"

partial def parseFields (json : Json) : Except String (List (String × Value)) := do
  let obj ← json.getObj?
  obj.toList.mapM (fun (k, v) => do
    let val ← fromJson v
    pure (k, val))

end

instance : FromJson Value where
  fromJson? := fromJson

end Value

namespace Pattern

partial def toJson : Pattern → Json
  | .wildcard => Json.mkObj [("type", "wildcard")]
  | .variable name => Json.mkObj [("type", "variable"), ("name", name)]
  | .literal v => Json.mkObj [("type", "literal"), ("value", v.toJson)]
  | .variant name fields =>
      Json.mkObj [("type", "variant"), ("name", name),
                  ("fields", Json.arr (fields.map (fun (n, p) =>
                    Json.mkObj [("name", n), ("pattern", p.toJson)])).toArray)]
  | .tuple elems =>
      Json.mkObj [("type", "tuple"), ("elements", Json.arr (elems.map toJson).toArray)]
  | .record fields =>
      Json.mkObj [("type", "record"),
                  ("fields", Json.arr (fields.map (fun (n, p) =>
                    Json.mkObj [("name", n), ("pattern", p.toJson)])).toArray)]

instance : ToJson Pattern where
  toJson := toJson

partial def fromJson (json : Json) : Except String Pattern := do
  let type ← json.getObjValAs? String "type"
  match type with
  | "wildcard" => pure .wildcard
  | "variable" =>
      let name ← json.getObjValAs? String "name"
      pure (.variable name)
  | "literal" =>
      let v ← json.getObjValAs? Value "value"
      pure (.literal v)
  | "variant" =>
      let name ← json.getObjValAs? String "name"
      let fieldsJson ← json.getObjVal? "fields"
      let fieldsArr ← fieldsJson.getArr?
      let fields ← fieldsArr.toList.mapM (fun j => do
        let n ← j.getObjValAs? String "name"
        let pJson ← j.getObjVal? "pattern"
        let p ← Pattern.fromJson pJson
        pure (n, p))
      pure (.variant name fields)
  | "tuple" =>
      let arr ← json.getObjVal? "elements"
      let elems ← arr.getArr?
      let ps ← elems.mapM fromJson
      pure (.tuple ps.toList)
  | "record" =>
      let fieldsJson ← json.getObjVal? "fields"
      let fieldsArr ← fieldsJson.getArr?
      let fields ← fieldsArr.toList.mapM (fun j => do
        let n ← j.getObjValAs? String "name"
        let pJson ← j.getObjVal? "pattern"
        let p ← Pattern.fromJson pJson
        pure (n, p))
      pure (.record fields)
  | _ => throw s!"Unknown pattern type: {type}"

instance : FromJson Pattern where
  fromJson? := fromJson

end Pattern

namespace Expr

partial def toJson : Expr → Json
  | .literal v => Json.mkObj [("type", "literal"), ("value", v.toJson)]
  | .variable name => Json.mkObj [("type", "variable"), ("name", name)]
  | .constructor name fields =>
      Json.mkObj [("type", "constructor"), ("name", name),
                  ("fields", Json.arr (fields.map (fun (n, e) =>
                    Json.mkObj [("name", n), ("expr", e.toJson)])).toArray)]
  | .tuple elems =>
      Json.mkObj [("type", "tuple"), ("elements", Json.arr (elems.map toJson).toArray)]
  | .match scrutinee arms =>
      Json.mkObj [("type", "match"), ("scrutinee", toJson scrutinee),
                  ("arms", Json.arr (arms.map (fun arm => 
                    Json.mkObj [("pattern", arm.pattern.toJson), ("body", arm.body.toJson)])).toArray)]
  | .if_let pattern expr then_branch else_branch =>
      Json.mkObj [("type", "if_let"), ("pattern", pattern.toJson),
                  ("expr", toJson expr), ("then_branch", toJson then_branch),
                  ("else_branch", toJson else_branch)]

instance : ToJson Expr where
  toJson := toJson

partial def fromJson (json : Json) : Except String Expr := do
  let type ← json.getObjValAs? String "type"
  match type with
  | "literal" =>
      let v ← json.getObjValAs? Value "value"
      pure (.literal v)
  | "variable" =>
      let name ← json.getObjValAs? String "name"
      pure (.variable name)
  | "constructor" =>
      let name ← json.getObjValAs? String "name"
      let fieldsJson ← json.getObjVal? "fields"
      let fieldsArr ← fieldsJson.getArr?
      let fields ← fieldsArr.toList.mapM (fun j => do
        let n ← j.getObjValAs? String "name"
        let eJson ← j.getObjVal? "expr"
        let e ← Expr.fromJson eJson
        pure (n, e))
      pure (.constructor name fields)
  | "tuple" =>
      let arr ← json.getObjVal? "elements"
      let elems ← arr.getArr?
      let es ← elems.mapM fromJson
      pure (.tuple es.toList)
  | "match" =>
      let scrutJson ← json.getObjVal? "scrutinee"
      let scrutinee ← Expr.fromJson scrutJson
      let armsJson ← json.getObjVal? "arms"
      let armsArr ← armsJson.getArr?
      let arms ← armsArr.toList.mapM (fun j => do
        let patJson ← j.getObjVal? "pattern"
        let pattern ← Pattern.fromJson patJson
        let bodyJson ← j.getObjVal? "body"
        let body ← Expr.fromJson bodyJson
        pure { pattern := pattern, body := body : MatchArm })
      pure (.match scrutinee arms)
  | "if_let" =>
      let patJson ← json.getObjVal? "pattern"
      let pattern ← Pattern.fromJson patJson
      let exprJson ← json.getObjVal? "expr"
      let expr ← Expr.fromJson exprJson
      let thenJson ← json.getObjVal? "then_branch"
      let then_branch ← Expr.fromJson thenJson
      let elseJson ← json.getObjVal? "else_branch"
      let else_branch ← Expr.fromJson elseJson
      pure (.if_let pattern expr then_branch else_branch)
  | _ => throw s!"Unknown expr type: {type}"

instance : FromJson Expr where
  fromJson? := fromJson

end Expr

namespace TypeExpr

def toJson : TypeExpr → Json
  | .named name => Json.mkObj [("type", "named"), ("name", name)]
  | .var id => Json.mkObj [("type", "var"), ("id", id)]
  | .constructor name args =>
      Json.mkObj [("type", "constructor"), ("name", name),
                  ("args", Json.arr (args.map toJson).toArray)]

instance : ToJson TypeExpr where
  toJson := toJson

partial def fromJson (json : Json) : Except String TypeExpr := do
  let type ← json.getObjValAs? String "type"
  match type with
  | "named" =>
      let name ← json.getObjValAs? String "name"
      pure (.named name)
  | "var" =>
      let id ← json.getObjValAs? Nat "id"
      pure (.var id)
  | "constructor" =>
      let name ← json.getObjValAs? String "name"
      let argsJson ← json.getObjVal? "args"
      let argsArr ← argsJson.getArr?
      let args ← argsArr.mapM fromJson
      pure (.constructor name args.toList)
  | _ => throw s!"Unknown type expression: {type}"

instance : FromJson TypeExpr where
  fromJson? := fromJson

end TypeExpr

namespace Variant

def toJson (v : Variant) : Json :=
  Json.mkObj [("name", v.name),
              ("fields", Json.arr (v.fields.map (fun (n, t) =>
                Json.mkObj [("name", n), ("type", t.toJson)])).toArray)]

instance : ToJson Variant where
  toJson := toJson

def fromJson (json : Json) : Except String Variant := do
  let name ← json.getObjValAs? String "name"
  let fieldsJson ← json.getObjVal? "fields"
  let fieldsArr ← fieldsJson.getArr?
  let fields ← fieldsArr.toList.mapM (fun j => do
    let n ← j.getObjValAs? String "name"
    let tJson ← j.getObjVal? "type"
    let t ← TypeExpr.fromJson tJson
    pure (n, t))
  pure { name := name, fields := fields }

instance : FromJson Variant where
  fromJson? := fromJson

end Variant

namespace TypeBody

def toJson : TypeBody → Json
  | .enum variants => Json.mkObj [("type", "enum"), ("variants", Json.arr (variants.map Variant.toJson).toArray)]
  | .struct fields => Json.mkObj [("type", "struct"),
                                   ("fields", Json.arr (fields.map (fun (n, t) =>
                                     Json.mkObj [("name", n), ("type", t.toJson)])).toArray)]

instance : ToJson TypeBody where
  toJson := toJson

def fromJson (json : Json) : Except String TypeBody := do
  let type ← json.getObjValAs? String "type"
  match type with
  | "enum" =>
      let variantsJson ← json.getObjVal? "variants"
      let variantsArr ← variantsJson.getArr?
      let variants ← variantsArr.mapM Variant.fromJson
      pure (.enum variants.toList)
  | "struct" =>
      let fieldsJson ← json.getObjVal? "fields"
      let fieldsArr ← fieldsJson.getArr?
      let fields ← fieldsArr.toList.mapM (fun j => do
        let n ← j.getObjValAs? String "name"
        let tJson ← j.getObjVal? "type"
        let t ← TypeExpr.fromJson tJson
        pure (n, t))
      pure (.struct fields)
  | _ => throw s!"Unknown type body: {type}"

instance : FromJson TypeBody where
  fromJson? := fromJson

end TypeBody

namespace TypeDef

def toJson (td : TypeDef) : Json :=
  Json.mkObj [("name", td.name), ("params", Json.arr (td.params.map (fun (n : Nat) => (n : Json))).toArray), ("body", td.body.toJson)]

instance : ToJson TypeDef where
  toJson := toJson

def fromJson (json : Json) : Except String TypeDef := do
  let name ← json.getObjValAs? String "name"
  let paramsJson ← json.getObjVal? "params"
  let paramsArr ← paramsJson.getArr?
  let params ← paramsArr.mapM (fun j => do
    let n ← j.getNat?
    pure n)
  let bodyJson ← json.getObjVal? "body"
  let body ← TypeBody.fromJson bodyJson
  pure { name := name, params := params.toList, body := body }

instance : FromJson TypeDef where
  fromJson? := fromJson

end TypeDef

/- ## Test Helpers -/

-- Helper to check JSON roundtrip
def testRoundtrip (v : Value) : Bool :=
  match Value.fromJson v.toJson with
  | .ok v' => v == v'
  | .error _ => false

end Ash
