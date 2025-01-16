(function() {
    var implementors = Object.fromEntries([["ink",[]],["ink_storage",[["impl&lt;K, V, Key, InnerKey&gt; StorableHint&lt;Key&gt; for <a class=\"struct\" href=\"ink_storage/struct.Mapping.html\" title=\"struct ink_storage::Mapping\">Mapping</a>&lt;K, V, InnerKey&gt;<div class=\"where\">where\n    V: Packed,\n    Key: StorageKey,\n    InnerKey: StorageKey,</div>"],["impl&lt;V, Key, InnerKey&gt; StorableHint&lt;Key&gt; for <a class=\"struct\" href=\"ink_storage/struct.Lazy.html\" title=\"struct ink_storage::Lazy\">Lazy</a>&lt;V, InnerKey&gt;<div class=\"where\">where\n    Key: StorageKey,\n    InnerKey: StorageKey,\n    V: StorableHint&lt;Key&gt;,</div>"],["impl&lt;V, Key, InnerKey&gt; StorableHint&lt;Key&gt; for <a class=\"struct\" href=\"ink_storage/struct.StorageVec.html\" title=\"struct ink_storage::StorageVec\">StorageVec</a>&lt;V, InnerKey&gt;<div class=\"where\">where\n    V: Packed,\n    Key: StorageKey,\n    InnerKey: StorageKey,</div>"]]],["ink_storage_traits",[]]]);
    if (window.register_implementors) {
        window.register_implementors(implementors);
    } else {
        window.pending_implementors = implementors;
    }
})()
//{"start":57,"fragment_lengths":[10,905,26]}