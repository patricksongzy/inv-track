<template>
  <Suspense>
    <template #default>
      <location-data />
    </template>
    <template #fallback>
      <p>Unable to load location.</p>
    </template>
  </Suspense>
</template>
<script>
import LocationData from '../components/LocationData.vue';

export default {
  components: {
    LocationData,
  },
};
</script>