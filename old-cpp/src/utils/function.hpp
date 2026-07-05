#ifndef UTILS_FUNCTION_HPP
#define UTILS_FUNCTION_HPP

#include <cstddef>

namespace Utils {
  namespace Function {
    template<std::size_t ...I> struct indices {};
    template<std::size_t N, std::size_t ...I> struct make_indices : make_indices<N - 1, N - 1, I...> {};
    template<std::size_t ...I> struct make_indices<0, I...> : indices<I...> {};

    // https://liam.page/2019/07/04/unpack-vector-as-parameters-for-functions/
    template<typename RetT, typename ...Args> struct fn_traits_def {
      static constexpr std::size_t arg_cnt = sizeof...(Args);
      using return_type = RetT;
      template<std::size_t i> struct arg {
        using type = typename std::tuple_element<i, std::tuple<Args...>>::type;
      };
    };
    template<typename T> struct fn_traits_impl;
    template<typename RetT, typename ...Args> 
    struct fn_traits_impl<RetT(Args...)> : fn_traits_def<RetT, Args...> {};
    template<typename RetT, typename ...Args>
    struct fn_traits_impl<RetT(*)(Args...)> : fn_traits_def<RetT, Args...> {};
    template<typename T> struct fn_traits : fn_traits_impl<T> {};
    // only accept function type: <number_t(number_t, number_t, ...)>
    template<typename FnType, 
             typename VecType, 
             std::size_t ...I, 
             typename Traits = fn_traits<FnType>,
             typename RetType = typename Traits::return_type>
    inline RetType call_func_with_indices(FnType& func, VecType& args, indices<I...>) {
      return func(*args[I]...);
    }
    template<typename FnType, 
             typename VecType, 
             typename Traits = fn_traits<FnType>,
             typename RetType = typename Traits::return_type>
    inline RetType call_func(FnType& func, VecType& args) {
      return call_func_with_indices(func, args, make_indices<Traits::arg_cnt>());
    }
  }
}

#endif