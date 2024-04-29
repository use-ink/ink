(function() {var type_impls = {
"ink_e2e":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-TransactAsset-for-FungiblesAdapter%3CAssets,+Matcher,+AccountIdConverter,+AccountId,+CheckAsset,+CheckingAccount%3E\" class=\"impl\"><a href=\"#impl-TransactAsset-for-FungiblesAdapter%3CAssets,+Matcher,+AccountIdConverter,+AccountId,+CheckAsset,+CheckingAccount%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;Assets, Matcher, AccountIdConverter, AccountId, CheckAsset, CheckingAccount&gt; TransactAsset for FungiblesAdapter&lt;Assets, Matcher, AccountIdConverter, AccountId, CheckAsset, CheckingAccount&gt;<span class=\"where fmt-newline\">where\n    Assets: Mutate&lt;AccountId&gt;,\n    Matcher: MatchesFungibles&lt;&lt;Assets as Inspect&lt;AccountId&gt;&gt;::AssetId, &lt;Assets as Inspect&lt;AccountId&gt;&gt;::Balance&gt;,\n    AccountIdConverter: ConvertLocation&lt;AccountId&gt;,\n    AccountId: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.75.0/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.75.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,\n    CheckAsset: AssetChecking&lt;&lt;Assets as Inspect&lt;AccountId&gt;&gt;::AssetId&gt;,\n    CheckingAccount: Get&lt;AccountId&gt;,</span></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.can_check_in\" class=\"method trait-impl\"><a href=\"#method.can_check_in\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">can_check_in</a>(\n    origin: &amp;Location,\n    what: &amp;Asset,\n    context: &amp;XcmContext\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.75.0/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.unit.html\">()</a>, Error&gt;</h4></section></summary><div class='docblock'>Ensure that <code>check_in</code> will do as expected. <a>Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.check_in\" class=\"method trait-impl\"><a href=\"#method.check_in\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">check_in</a>(origin: &amp;Location, what: &amp;Asset, context: &amp;XcmContext)</h4></section></summary><div class='docblock'>An asset has been teleported in from the given origin. This should do whatever housekeeping\nis needed. <a>Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.can_check_out\" class=\"method trait-impl\"><a href=\"#method.can_check_out\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">can_check_out</a>(\n    dest: &amp;Location,\n    what: &amp;Asset,\n    context: &amp;XcmContext\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.75.0/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.unit.html\">()</a>, Error&gt;</h4></section></summary><div class='docblock'>Ensure that <code>check_out</code> will do as expected. <a>Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.check_out\" class=\"method trait-impl\"><a href=\"#method.check_out\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">check_out</a>(dest: &amp;Location, what: &amp;Asset, context: &amp;XcmContext)</h4></section></summary><div class='docblock'>An asset has been teleported out to the given destination. This should do whatever\nhousekeeping is needed. <a>Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.deposit_asset\" class=\"method trait-impl\"><a href=\"#method.deposit_asset\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">deposit_asset</a>(\n    what: &amp;Asset,\n    who: &amp;Location,\n    context: <a class=\"enum\" href=\"https://doc.rust-lang.org/1.75.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;&amp;XcmContext&gt;\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.75.0/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.unit.html\">()</a>, Error&gt;</h4></section></summary><div class='docblock'>Deposit the <code>what</code> asset into the account of <code>who</code>. <a>Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.withdraw_asset\" class=\"method trait-impl\"><a href=\"#method.withdraw_asset\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">withdraw_asset</a>(\n    what: &amp;Asset,\n    who: &amp;Location,\n    maybe_context: <a class=\"enum\" href=\"https://doc.rust-lang.org/1.75.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;&amp;XcmContext&gt;\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.75.0/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;AssetsInHolding, Error&gt;</h4></section></summary><div class='docblock'>Withdraw the given asset from the consensus system. Return the actual asset(s) withdrawn,\nwhich should always be equal to <code>_what</code>. <a>Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.internal_transfer_asset\" class=\"method trait-impl\"><a href=\"#method.internal_transfer_asset\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">internal_transfer_asset</a>(\n    what: &amp;Asset,\n    from: &amp;Location,\n    to: &amp;Location,\n    context: &amp;XcmContext\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.75.0/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;AssetsInHolding, Error&gt;</h4></section></summary><div class='docblock'>Move an <code>asset</code> <code>from</code> one location in <code>to</code> another location. <a>Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.transfer_asset\" class=\"method trait-impl\"><a href=\"#method.transfer_asset\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">transfer_asset</a>(\n    asset: &amp;Asset,\n    from: &amp;Location,\n    to: &amp;Location,\n    context: &amp;XcmContext\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.75.0/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;AssetsInHolding, Error&gt;</h4></section></summary><div class='docblock'>Move an <code>asset</code> <code>from</code> one location in <code>to</code> another location. <a>Read more</a></div></details></div></details>","TransactAsset","ink_e2e::preset::mock_network::parachain::ForeignAssetsTransactor"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()