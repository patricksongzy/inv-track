<template>
  <h1>Locations</h1>
  <Suspense>
    <template #default>
      <locations-list />
    </template>
    <template #fallback>
      <p>Unable to load locations.</p>
    </template>
  </Suspense>
</template>
<script>
import LocationsList from '../components/LocationsList.vue';

export default {
  components: {
    LocationsList,
  },
};
</script>