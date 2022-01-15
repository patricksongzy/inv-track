<template>
  <div v-if="live?.length > 0 || base?.length > 0">
    <table>
      <tr>
        <th>ID</th>
        <th>SKU</th>
        <th>Name</th>
        <th>Quantity</th>
      </tr>
      <tr v-for="item in live ?? base" :key="item.id">
        <td>{{ item.id }}</td>
        <td>{{ item.sku ?? '--' }}</td>
        <td>{{ item.name }}</td>
        <td>{{ item.quantity }}</td>
      </tr>
    </table>
  </div>
  <div v-else>
    <p>No items added to the inventory tracking system.</p>
  </div>
</template>
<script>
import { useClientHandle } from '@urql/vue';

export default {
  async setup() {
    const handle = useClientHandle();

    // get the initial items
    let result = await handle.useQuery({
      query: `
      {
        items {
          id,
          sku,
          name,
          quantity,
        }
      }
      `
    }).executeQuery();
    result = JSON.parse(JSON.stringify(result.data.value.items));

    // update the items live
    const handleSubscription = (items = result, response) => {
      const event = response.itemSubscription;
      if (event.modification === 'CREATE') {
        return [...items, event.data];
      } else if (event.modification === 'UPDATE') {
        const foundIndex = items.findIndex(item => item.id === event.data.id);
        items[foundIndex] = event.data;
        return items;
      } else if (event.modification === 'DELETE') {
        return items.filter(item => item.id !== event.data.id);
      }
    }

    // subscribe to item changes
    const subscription = handle.useSubscription({
      query: `
      subscription {
        itemSubscription {
          modification,
          data {
            id,
            sku,
            name,
            quantity
          }
        }
      }
      `
    }, handleSubscription);

    // hack to get things working
    return {
      base: result,
      live: subscription.data,
    };
  }
};
</script>