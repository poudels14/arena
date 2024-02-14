import ast


# remove the last statement from the code if it's a Expr
# such that expr can be evaluated by itself to get the result
# of the code in the rust
def pop_last_expr(code):
    astt = ast.parse(code)
    if isinstance(astt.body[-1], ast.Expr):
        return [ast.unparse(astt.body[:-1]), ast.unparse(astt.body[-1])]
    return [code]
