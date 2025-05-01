(function() {
    var implementors = Object.fromEntries([["ink",[]],["ink_env",[["impl SolEncode&lt;'_&gt; for <a class=\"type\" href=\"ink_env/call/utils/type.EmptyArgumentList.html\" title=\"type ink_env::call::utils::EmptyArgumentList\">EmptyArgumentList</a>&lt;<a class=\"struct\" href=\"ink_env/reflect/struct.SolEncoding.html\" title=\"struct ink_env::reflect::SolEncoding\">SolEncoding</a>&gt;"],["impl&lt;'a, Head, Rest&gt; SolEncode&lt;'a&gt; for <a class=\"struct\" href=\"ink_env/call/utils/struct.ArgumentList.html\" title=\"struct ink_env::call::utils::ArgumentList\">ArgumentList</a>&lt;<a class=\"struct\" href=\"ink_env/call/utils/struct.Argument.html\" title=\"struct ink_env::call::utils::Argument\">Argument</a>&lt;Head&gt;, Rest, <a class=\"struct\" href=\"ink_env/reflect/struct.SolEncoding.html\" title=\"struct ink_env::reflect::SolEncoding\">SolEncoding</a>&gt;<div class=\"where\">where\n    Head: SolEncode&lt;'a&gt;,\n    Rest: SolEncode&lt;'a&gt;,</div>"],["impl&lt;'a, T&gt; SolEncode&lt;'a&gt; for <a class=\"struct\" href=\"ink_env/call/utils/struct.Argument.html\" title=\"struct ink_env::call::utils::Argument\">Argument</a>&lt;T&gt;<div class=\"where\">where\n    T: SolEncode&lt;'a&gt;,</div>"]]],["ink_primitives",[]]]);
    if (window.register_implementors) {
        window.register_implementors(implementors);
    } else {
        window.pending_implementors = implementors;
    }
})()
//{"start":57,"fragment_lengths":[10,1164,22]}