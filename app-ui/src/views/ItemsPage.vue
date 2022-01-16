<template>
  <h1 class="title">Items</h1>
  <Suspense>
    <template #default>
      <items-list />
    </template>
    <template #fallback>
      <p>Unable to load items.</p>
    </template>
  </Suspense>
  <hr />
  <h1 class="subtitle">Create Item</h1>
  <item-change />
</template>
<script>
import ItemsList from '../components/ItemsList.vue';
import ItemChange from '../components/ItemChange.vue';

export default {
  components: {
    ItemsList,
    ItemChange,
  },
};
</script>