(function() {
    var implementors = Object.fromEntries([["ink",[]],["ink_storage",[["impl&lt;K, V, KeyType&gt; StorageLayout for <a class=\"struct\" href=\"ink_storage/struct.Mapping.html\" title=\"struct ink_storage::Mapping\">Mapping</a>&lt;K, V, KeyType&gt;<div class=\"where\">where\n    K: TypeInfo + 'static,\n    V: Packed + StorageLayout + TypeInfo + 'static,\n    KeyType: StorageKey + TypeInfo + 'static,</div>"],["impl&lt;V, KeyType&gt; StorageLayout for <a class=\"struct\" href=\"ink_storage/struct.Lazy.html\" title=\"struct ink_storage::Lazy\">Lazy</a>&lt;V, KeyType&gt;<div class=\"where\">where\n    V: StorageLayout + TypeInfo + 'static,\n    KeyType: StorageKey + TypeInfo + 'static,</div>"],["impl&lt;V, KeyType&gt; StorageLayout for <a class=\"struct\" href=\"ink_storage/struct.StorageVec.html\" title=\"struct ink_storage::StorageVec\">StorageVec</a>&lt;V, KeyType&gt;<div class=\"where\">where\n    V: Packed + StorageLayout + TypeInfo + 'static,\n    KeyType: StorageKey + TypeInfo + 'static,</div>"]]],["ink_storage_traits",[]]]);
    if (window.register_implementors) {
        window.register_implementors(implementors);
    } else {
        window.pending_implementors = implementors;
    }
})()
//{"start":57,"fragment_lengths":[10,961,26]}