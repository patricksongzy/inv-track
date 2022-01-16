<template>
  <div v-if="items.length > 0">
    <div class="table-container">
      <table class="table is-fullwidth is-hoverable">
        <thead>
          <tr>
            <th>ID</th>
            <th>SKU</th>
            <th>Name</th>
            <th>Supplier</th>
            <th>Quantity</th>
            <th>Link</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="item in items" :key="item.id">
            <td>{{ item.id }}</td>
            <td>{{ item.sku ?? '--' }}</td>
            <td>{{ item.name }}</td>
            <td>{{ item.supplier ?? '--' }}</td>
            <td>{{ item.quantity }}</td>
            <td>
              <router-link
                v-bind:to="`/items/${item.id}`"
                class="button is-link"
                >View Item</router-link
              >
            </td>
          </tr>
        </tbody>
      </table>
    </div>
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
    let result = await handle
      .useQuery({
        query: `
      {
        items {
          id,
          sku,
          name,
          supplier,
          quantity,
        }
      }
      `,
      })
      .executeQuery();
    result = JSON.parse(JSON.stringify(result.data.value.items));

    // update the items live
    const handleSubscription = (items = result, response) => {
      const event = response.itemSubscription;
      if (event.modification === 'CREATE') {
        return [...items, event.data];
      } else if (event.modification === 'UPDATE') {
        const foundIndex = items.findIndex((item) => item.id === event.data.id);
        items[foundIndex] = event.data;
        return items;
      } else if (event.modification === 'DELETE') {
        return items.filter((item) => item.id !== event.data.id);
      }
    };

    // subscribe to item changes
    const subscription = handle.useSubscription(
      {
        query: `
      subscription {
        itemSubscription {
          modification,
          data {
            id,
            sku,
            name,
            supplier,
            quantity
          }
        }
      }
      `,
      },
      handleSubscription
    );

    // hack to get things working
    return {
      base: result,
      live: subscription.data,
    };
  },
  computed: {
    items: function () {
      return this.live ?? this.base;
    },
  },
};
</script>