<template>
  <Suspense>
    <template #default>
      <transaction-data />
    </template>
    <template #fallback>
      <p>Unable to load transaction.</p>
    </template>
  </Suspense>
</template>
<script>
import TransactionData from '../components/TransactionData.vue';

export default {
  components: {
    TransactionData,
  },
};
</script>