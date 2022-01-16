<template>
  <h1 class="title">Locations</h1>
  <Suspense>
    <template #default>
      <locations-list />
    </template>
    <template #fallback>
      <p>Unable to load locations.</p>
    </template>
  </Suspense>
  <hr />
  <h1 class="subtitle">Create Location</h1>
  <location-change />
</template>
<script>
import LocationsList from '../components/LocationsList.vue';
import LocationChange from '../components/LocationChange.vue';

export default {
  components: {
    LocationsList,
    LocationChange,
  },
};
</script>