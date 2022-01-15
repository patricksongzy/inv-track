<template>
  <h1>Items</h1>
  <Suspense>
    <template #default>
      <items-list />
    </template>
    <template #fallback>
      <p>Unable to load items.</p>
    </template>
  </Suspense>
</template>
<script>
import ItemsList from '../components/ItemsList.vue';

export default {
  components: {
    ItemsList,
  },
};
</script>