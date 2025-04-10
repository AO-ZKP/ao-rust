#ifndef ADDITION_H
#define ADDITION_H

#include <lauxlib.h>
#include <lua.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

// Original addition.h declarations
int32_t add(int32_t a, int32_t b);

// Lua bindings
static int l_add(lua_State *L) {
  lua_Integer a = luaL_checkinteger(L, 1); // Get first argument as integer
  lua_Integer b = luaL_checkinteger(L, 2); // Get second argument as integer
  lua_Integer result = add(a, b);          // Call the C add function
  lua_pushinteger(L, result);              // Push result to Lua stack
  return 1;                                // Return one value
}

static const struct luaL_Reg addition_functions[] = {
    {"add", l_add}, // Map Lua function name "add" to l_add
    {NULL, NULL}    // Sentinel to end the array
};

int luaopen_addition(lua_State *L) {
  luaL_newlib(L, addition_functions); // Register the functions with Lua
  return 1;                           // Return one value (the module table)
}

#endif // ADDITION_H
