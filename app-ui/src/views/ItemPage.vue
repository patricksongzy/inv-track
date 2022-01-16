<template>
  <Suspense>
    <template #default>
      <item-data />
    </template>
    <template #fallback>
      <p>Unable to load item.</p>
    </template>
  </Suspense>
</template>
<script>
import ItemData from '../components/ItemData.vue';

export default {
  components: {
    ItemData,
  },
};
</script>