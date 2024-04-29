(function() {var type_impls = {
"ink_e2e":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-SendXcm-for-ParachainXcmRouter%3CT%3E\" class=\"impl\"><a href=\"#impl-SendXcm-for-ParachainXcmRouter%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T&gt; SendXcm for <a class=\"struct\" href=\"ink_e2e/preset/mock_network/struct.ParachainXcmRouter.html\" title=\"struct ink_e2e::preset::mock_network::ParachainXcmRouter\">ParachainXcmRouter</a>&lt;T&gt;<span class=\"where fmt-newline\">where\n    T: Get&lt;Id&gt;,</span></h3></section></summary><div class=\"impl-items\"><details class=\"toggle\" open><summary><section id=\"associatedtype.Ticket\" class=\"associatedtype trait-impl\"><a href=\"#associatedtype.Ticket\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a class=\"associatedtype\">Ticket</a> = (Id, Location, Xcm&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.unit.html\">()</a>&gt;)</h4></section></summary><div class='docblock'>Intermediate value which connects the two phases of the send operation.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.validate\" class=\"method trait-impl\"><a href=\"#method.validate\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">validate</a>(\n    destination: &amp;mut <a class=\"enum\" href=\"https://doc.rust-lang.org/1.75.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;Location&gt;,\n    message: &amp;mut <a class=\"enum\" href=\"https://doc.rust-lang.org/1.75.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;Xcm&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.unit.html\">()</a>&gt;&gt;\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.75.0/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;((Id, Location, Xcm&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.unit.html\">()</a>&gt;), Assets), SendError&gt;</h4></section></summary><div class='docblock'>Check whether the given <code>_message</code> is deliverable to the given <code>_destination</code> and if so\ndetermine the cost which will be paid by this chain to do so, returning a <code>Validated</code> token\nwhich can be used to enact delivery. <a>Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.deliver\" class=\"method trait-impl\"><a href=\"#method.deliver\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">deliver</a>(triple: (Id, Location, Xcm&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.unit.html\">()</a>&gt;)) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.75.0/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;[<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.u8.html\">u8</a>; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.array.html\">32</a>], SendError&gt;</h4></section></summary><div class='docblock'>Actually carry out the delivery operation for a previously validated message sending.</div></details></div></details>","SendXcm","ink_e2e::preset::mock_network::parachain::XcmRouter"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()